// Imports
use crate::config;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

pub(crate) fn lib_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let exec_dir = exec_parent_dir()?;
        let candidate = exec_dir.join("..\\lib");
        if candidate.exists() {
            Ok(candidate)
        } else {
            let msys_lib = PathBuf::from("C:\\msys64\\ucrt64\\lib");
            if msys_lib.exists() {
                Ok(msys_lib)
            } else {
                Ok(candidate)
            }
        }
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(&canonicalized_exec_dir) {
            Ok(canonicalized_exec_dir.join("../Frameworks"))
        } else {
            Ok(PathBuf::from(config::LIBDIR))
        }
    } else {
        Ok(PathBuf::from(config::LIBDIR))
    }
}

pub(crate) fn data_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let exec_dir = exec_parent_dir()?;
        let candidate = exec_dir.join("..\\share");
        if candidate.exists() {
            Ok(candidate)
        } else {
            let msys_share = PathBuf::from("C:\\msys64\\ucrt64\\share");
            if msys_share.exists() {
                Ok(msys_share)
            } else {
                Ok(candidate)
            }
        }
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(&canonicalized_exec_dir) {
            Ok(canonicalized_exec_dir.join("../Resources/share"))
        } else {
            Ok(PathBuf::from(config::DATADIR))
        }
    } else {
        Ok(PathBuf::from(config::DATADIR))
    }
}

pub(crate) fn pkg_data_dir() -> anyhow::Result<PathBuf> {
    Ok(data_dir()?.join(config::APP_NAME))
}

pub(crate) fn locale_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let exec_dir = exec_parent_dir()?;
        let candidate = exec_dir.join("..\\share\\locale");
        if candidate.exists() {
            Ok(candidate)
        } else {
            let msys_locale = PathBuf::from("C:\\msys64\\ucrt64\\share\\locale");
            if msys_locale.exists() {
                Ok(msys_locale)
            } else {
                Ok(candidate)
            }
        }
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(&canonicalized_exec_dir) {
            Ok(canonicalized_exec_dir.join("../Resources/share/locale"))
        } else {
            Ok(PathBuf::from(config::LOCALEDIR))
        }
    } else {
        Ok(PathBuf::from(config::LOCALEDIR))
    }
}

/// depending on the target platform we need to set some env vars on startup
pub(crate) fn setup_env() -> anyhow::Result<()> {
    if cfg!(target_os = "windows") {
        let data_dir = data_dir()?;
        let lib_dir = lib_dir()?;
        let exec_dir = exec_parent_dir()?;

        // SAFETY: this setup only happens while still being single-threaded
        unsafe {
            let mut xdg_data_dirs = Vec::new();
            if data_dir.exists() {
                xdg_data_dirs.push(data_dir);
            }
            let msys_share = PathBuf::from("C:\\msys64\\ucrt64\\share");
            if msys_share.exists() && !xdg_data_dirs.contains(&msys_share) {
                xdg_data_dirs.push(msys_share);
            }
            if let Some(existing) = std::env::var_os("XDG_DATA_DIRS") {
                xdg_data_dirs.extend(std::env::split_paths(&existing));
            }
            if let Ok(joined) = std::env::join_paths(xdg_data_dirs) {
                std::env::set_var("XDG_DATA_DIRS", joined);
            }

            if std::env::var_os("GSETTINGS_SCHEMA_DIR").is_none() {
                let mut schema_dirs = Vec::new();
                let build_schemas = exec_dir.join("schemas");
                if build_schemas.join("gschemas.compiled").exists() {
                    schema_dirs.push(build_schemas);
                }
                let target_schemas = exec_dir.join("..\\..\\schemas");
                if target_schemas.join("gschemas.compiled").exists() {
                    schema_dirs.push(target_schemas);
                }
                let msys_schemas = PathBuf::from("C:\\msys64\\ucrt64\\share\\glib-2.0\\schemas");
                if msys_schemas.join("gschemas.compiled").exists() {
                    schema_dirs.push(msys_schemas);
                }

                if let Ok(joined) = std::env::join_paths(schema_dirs) {
                    if !joined.is_empty() {
                        std::env::set_var("GSETTINGS_SCHEMA_DIR", joined);
                    }
                }
            }

            std::env::set_var(
                "GDK_PIXBUF_MODULEDIR",
                lib_dir.join("gdk-pixbuf-2.0\\2.10.0\\loaders"),
            );
        }

        // The OpenGL renderer has proven to be buggy and can even produce crashes on windows.
        // Stick to vulkan by default.
        // If it's initialization fails, Gtk4 falls back to OpenGL anyway.
        if std::env::var_os("GSK_RENDERER").is_none() {
            unsafe { std::env::set_var("GSK_RENDERER", "vulkan") };
        }

        // Without DirectComposition GSK falls back to the cairo software
        // renderer, where strokes and images render as flat coloured (pink) boxes.
        if std::env::var_os("GDK_DEBUG").is_none() {
            unsafe { std::env::set_var("GDK_DEBUG", "dcomp") };
        }
        // On Windows, pangocairo renders glyphs through cairo's win32 font backend, which goes to DirectWrite
        // and Direct2D.
        // On Windows that path is broken: pango logs "All font fallbacks failed" for every layout and rendering
        // text eventually dies with an access violation (0xc0000005) in d2d1.dll.
        // It reproducibly takes down the app when a document containing a text stroke is opened, or when the
        // typewriter is used.
        //
        // The fontconfig/freetype backend works correctly here, so select it explicitly.
        // Fontconfig is present and configured in the MSYS2 environment the app is built and shipped with.
        //
        // This deliberately does not use `std::env::set_var`: on Windows that only calls SetEnvironmentVariableW,
        // which updates the Win32 environment block but not the copy the C runtime builds at startup.
        // pangocairo reads the variable with plain `getenv` (unlike GDK, which uses `g_getenv` and therefore does
        // see `set_var`), so it would never observe the value. `_putenv_s` updates the CRT table that `getenv`
        // reads, and pangocairo resolves to the same UCRT as the app.
        if std::env::var_os("PANGOCAIRO_BACKEND").is_none() {
            unsafe extern "C" {
                fn _putenv_s(
                    name: *const std::ffi::c_char,
                    value: *const std::ffi::c_char,
                ) -> std::ffi::c_int;
            }

            unsafe { _putenv_s(c"PANGOCAIRO_BACKEND".as_ptr(), c"fc".as_ptr()) };
        }

        //unsafe { std::env::set_var("RUST_LOG", "rnote=debug,rnote-cli=debug,rnote-engine=debug,rnote-compose=debug") };
    } else if cfg!(target_os = "macos") {
        let canonicalized_exec_dir = exec_parent_dir()?.canonicalize()?;

        if macos_is_in_app_bundle(canonicalized_exec_dir) {
            let data_dir = data_dir()?;
            let resources_dir = exec_parent_dir()?.canonicalize()?.join("../Resources");

            // SAFETY: this setup only happens while still being single-threaded
            unsafe {
                std::env::set_var("XDG_DATA_DIRS", data_dir);
                std::env::set_var(
                    "GDK_PIXBUF_MODULE_FILE",
                    resources_dir.join("etc/loaders.cache"),
                );
            }
        }
    }
    Ok(())
}

fn exec_parent_dir() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_exe()?
        .parent()
        .ok_or(anyhow::anyhow!(
            "could not get parent dir of executable path"
        ))?
        .to_path_buf())
}

// this returns true when the app is packaged as a relocatable application bundle
fn macos_is_in_app_bundle(canonicalized_exec_dir: impl AsRef<Path>) -> bool {
    canonicalized_exec_dir
        .as_ref()
        .components()
        .zip(canonicalized_exec_dir.as_ref().components().skip(1))
        .any(|(a, b)| {
            if let Component::Normal(a) = a
                && let Component::Normal(b) = b
            {
                a == OsStr::new("Contents") && b == OsStr::new("MacOS")
            } else {
                false
            }
        })
}
