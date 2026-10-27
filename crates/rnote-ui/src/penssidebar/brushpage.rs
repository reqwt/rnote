// Imports
use crate::{RnAppWindow, RnStrokeWidthPicker};
use adw::prelude::*;
use gtk4::{
    Button, CompositeTemplate, ListBox, MenuButton, Popover, ToggleButton, Widget, glib,
    glib::clone, subclass::prelude::*,
};
use num_traits::cast::ToPrimitive;
use rnote_compose::builders::PenPathBuilderType;
use rnote_compose::style::PressureCurve;
use rnote_compose::style::smooth::TipProfile;
use rnote_compose::style::textured::{TexturedDotsDistribution, TexturedOptions};
use rnote_engine::ext::GdkRGBAExt;
use rnote_engine::pens::pensconfig::BrushConfig;
use rnote_engine::pens::pensconfig::brushconfig::{BrushStyle, SolidOptions};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/brushpage.ui")]
    pub(crate) struct RnBrushPage {
        #[template_child]
        pub(crate) brushstyle_solid_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) brushstyle_marker_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) brushstyle_textured_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) brushstyle_chisel_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) brushconfig_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) brushconfig_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) brushconfig_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) brush_buildertype_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub(crate) brush_buildertype_simple: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) brush_buildertype_curved: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) brush_buildertype_modeled: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(crate) solidstyle_tip_profile_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) solidstyle_pressure_curves_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) chisel_options_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub(crate) chiselstyle_aspect_ratio_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) chiselstyle_fixed_angle_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) chiselstyle_tilt_responsive_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) texturedstyle_density_row: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub(crate) texturedstyle_distribution_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(crate) stroke_width_picker: TemplateChild<RnStrokeWidthPicker>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnBrushPage {
        const NAME: &'static str = "RnBrushPage";
        type Type = super::RnBrushPage;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnBrushPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnBrushPage {}
}

glib::wrapper! {
    pub(crate) struct RnBrushPage(ObjectSubclass<imp::RnBrushPage>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnBrushPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnBrushPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn brushconfig_menubutton(&self) -> MenuButton {
        self.imp().brushconfig_menubutton.get()
    }

    pub(crate) fn brush_style(&self) -> Option<BrushStyle> {
        if self.imp().brushstyle_marker_toggle.is_active() {
            Some(BrushStyle::Marker)
        } else if self.imp().brushstyle_solid_toggle.is_active() {
            Some(BrushStyle::Solid)
        } else if self.imp().brushstyle_textured_toggle.is_active() {
            Some(BrushStyle::Textured)
        } else if self.imp().brushstyle_chisel_toggle.is_active() {
            Some(BrushStyle::Chisel)
        } else {
            None
        }
    }

    pub(crate) fn set_brush_style(&self, brush_style: BrushStyle) {
        match brush_style {
            BrushStyle::Marker => self.imp().brushstyle_marker_toggle.set_active(true),
            BrushStyle::Solid => self.imp().brushstyle_solid_toggle.set_active(true),
            BrushStyle::Textured => self.imp().brushstyle_textured_toggle.set_active(true),
            BrushStyle::Chisel => self.imp().brushstyle_chisel_toggle.set_active(true),
        }
    }

    fn apply_brush_style_selection(&self, appwindow: &RnAppWindow, brush_style: BrushStyle) {
        appwindow
            .engine_config()
            .write()
            .pens_config
            .brush_config
            .style = brush_style;
        self.stroke_width_picker().deselect_setters();

        let active_stroke_color = appwindow
            .overlays()
            .colorpicker()
            .stroke_color()
            .into_compose_color();
        let active_fill_color = appwindow
            .overlays()
            .colorpicker()
            .fill_color()
            .into_compose_color();
        {
            let mut engine_config = appwindow.engine_config().write();
            engine_config
                .pens_config
                .set_all_stroke_colors(active_stroke_color);
            engine_config
                .pens_config
                .set_all_fill_colors(active_fill_color);
        }

        match brush_style {
            BrushStyle::Marker => {
                let stroke_width = appwindow
                    .engine_config()
                    .read()
                    .pens_config
                    .brush_config
                    .marker_options
                    .stroke_width;
                self.imp()
                    .stroke_width_picker
                    .set_stroke_width(stroke_width);
            }
            BrushStyle::Solid => {
                let stroke_width = appwindow
                    .engine_config()
                    .read()
                    .pens_config
                    .brush_config
                    .solid_options
                    .stroke_width;
                self.imp()
                    .stroke_width_picker
                    .set_stroke_width(stroke_width);
            }
            BrushStyle::Textured => {
                let stroke_width = appwindow
                    .engine_config()
                    .read()
                    .pens_config
                    .brush_config
                    .textured_options
                    .stroke_width;
                self.imp()
                    .stroke_width_picker
                    .set_stroke_width(stroke_width);
            }
            BrushStyle::Chisel => {
                let stroke_width = appwindow
                    .engine_config()
                    .read()
                    .pens_config
                    .brush_config
                    .chisel_options
                    .stroke_width;
                self.imp()
                    .stroke_width_picker
                    .set_stroke_width(stroke_width);
            }
        }
    }

    pub(crate) fn buildertype(&self) -> Option<PenPathBuilderType> {
        PenPathBuilderType::try_from(
            self.imp().brush_buildertype_listbox.selected_row()?.index() as u32
        )
        .ok()
    }

    pub(crate) fn set_buildertype(&self, buildertype: PenPathBuilderType) {
        match buildertype {
            PenPathBuilderType::Simple => self
                .imp()
                .brush_buildertype_listbox
                .select_row(Some(&*self.imp().brush_buildertype_simple)),
            PenPathBuilderType::Curved => self
                .imp()
                .brush_buildertype_listbox
                .select_row(Some(&*self.imp().brush_buildertype_curved)),
            PenPathBuilderType::Modeled => self
                .imp()
                .brush_buildertype_listbox
                .select_row(Some(&*self.imp().brush_buildertype_modeled)),
        }
    }

    pub(crate) fn solidstyle_pressure_curve(&self) -> PressureCurve {
        PressureCurve::try_from(self.imp().solidstyle_pressure_curves_row.get().selected()).unwrap()
    }

    pub(crate) fn set_solidstyle_pressure_curve(&self, pressure_curve: PressureCurve) {
        let position = pressure_curve.to_u32().unwrap();

        self.imp()
            .solidstyle_pressure_curves_row
            .get()
            .set_selected(position);
    }

    pub(crate) fn solidstyle_tip_profile(&self) -> TipProfile {
        match self.imp().solidstyle_tip_profile_row.get().selected() {
            1 => self.chisel_tip_profile(),
            _ => TipProfile::Circular,
        }
    }

    pub(crate) fn set_solidstyle_tip_profile(&self, tip_profile: TipProfile) {
        match tip_profile {
            TipProfile::Circular => {
                self.imp().solidstyle_tip_profile_row.get().set_selected(0);
            }
            TipProfile::Chisel {
                aspect_ratio,
                fixed_angle,
                use_stylus_tilt,
            } => {
                self.imp().solidstyle_tip_profile_row.get().set_selected(1);
                self.imp().chiselstyle_aspect_ratio_row.set_value(aspect_ratio);
                self.imp()
                    .chiselstyle_fixed_angle_row
                    .set_value(fixed_angle.to_degrees());
                self.imp()
                    .chiselstyle_tilt_responsive_row
                    .set_active(use_stylus_tilt);
            }
        }
    }

    pub(crate) fn chisel_tip_profile(&self) -> TipProfile {
        let aspect_ratio = self.imp().chiselstyle_aspect_ratio_row.value();
        let fixed_angle = self.imp().chiselstyle_fixed_angle_row.value().to_radians();
        let use_stylus_tilt = self.imp().chiselstyle_tilt_responsive_row.is_active();
        TipProfile::Chisel {
            aspect_ratio,
            fixed_angle,
            use_stylus_tilt,
        }
    }

    pub(crate) fn texturedstyle_dots_distribution(&self) -> TexturedDotsDistribution {
        TexturedDotsDistribution::try_from(
            self.imp().texturedstyle_distribution_row.get().selected(),
        )
        .unwrap()
    }

    pub(crate) fn set_texturedstyle_distribution_variant(
        &self,
        distribution: TexturedDotsDistribution,
    ) {
        let position = distribution.to_u32().unwrap();

        self.imp()
            .texturedstyle_distribution_row
            .get()
            .set_selected(position);
    }

    pub(crate) fn stroke_width_picker(&self) -> RnStrokeWidthPicker {
        self.imp().stroke_width_picker.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let brushconfig_popover = imp.brushconfig_popover.get();

        // Popovers
        imp.brushconfig_popover_close_button.connect_clicked(clone!(
            #[weak]
            brushconfig_popover,
            move |_| {
                brushconfig_popover.popdown();
            }
        ));

        // Stroke width
        imp.stroke_width_picker
            .spinbutton()
            .set_range(BrushConfig::STROKE_WIDTH_MIN, BrushConfig::STROKE_WIDTH_MAX);
        // set value after the range!
        imp.stroke_width_picker
            .set_stroke_width(SolidOptions::default().stroke_width);

        imp.stroke_width_picker.connect_notify_local(
            Some("stroke-width"),
            clone!(
                #[weak]
                appwindow,
                move |picker, _| {
                    let stroke_width = picker.stroke_width();
                    let brush_style = appwindow
                        .engine_config()
                        .read()
                        .pens_config
                        .brush_config
                        .style;

                    match brush_style {
                        BrushStyle::Marker => {
                            appwindow
                                .engine_config()
                                .write()
                                .pens_config
                                .brush_config
                                .marker_options
                                .stroke_width = stroke_width;
                        }
                        BrushStyle::Solid => {
                            appwindow
                                .engine_config()
                                .write()
                                .pens_config
                                .brush_config
                                .solid_options
                                .stroke_width = stroke_width;
                        }
                        BrushStyle::Textured => {
                            appwindow
                                .engine_config()
                                .write()
                                .pens_config
                                .brush_config
                                .textured_options
                                .stroke_width = stroke_width;
                        }
                        BrushStyle::Chisel => {
                            appwindow
                                .engine_config()
                                .write()
                                .pens_config
                                .brush_config
                                .chisel_options
                                .stroke_width = stroke_width;
                        }
                    }
                }
            ),
        );

        // Style
        imp.brushstyle_marker_toggle.connect_toggled(clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }

                let Some(brush_style) = brushpage.brush_style() else {
                    return;
                };

                brushpage.apply_brush_style_selection(&appwindow, brush_style);
            }
        ));

        imp.brushstyle_solid_toggle.connect_toggled(clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }

                let Some(brush_style) = brushpage.brush_style() else {
                    return;
                };

                brushpage.apply_brush_style_selection(&appwindow, brush_style);
            }
        ));

        imp.brushstyle_textured_toggle.connect_toggled(clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }

                let Some(brush_style) = brushpage.brush_style() else {
                    return;
                };

                brushpage.apply_brush_style_selection(&appwindow, brush_style);
            }
        ));

        imp.brushstyle_chisel_toggle.connect_toggled(clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }

                let Some(brush_style) = brushpage.brush_style() else {
                    return;
                };

                brushpage.apply_brush_style_selection(&appwindow, brush_style);
            }
        ));

        // Builder type
        imp.brush_buildertype_listbox.connect_row_selected(clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move |_, _| {
                let Some(buildertype) = brushpage.buildertype() else {
                    return;
                };
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .builder_type = buildertype;
            }
        ));

        // Solid style
        // Tip profile
        imp.solidstyle_tip_profile_row
            .get()
            .connect_selected_notify(clone!(
                #[weak(rename_to=brushpage)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    let tip_profile = brushpage.solidstyle_tip_profile();
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .brush_config
                        .solid_options
                        .tip_profile = tip_profile;
                }
            ));

        // Pressure curve
        imp.solidstyle_pressure_curves_row
            .get()
            .connect_selected_notify(clone!(
                #[weak(rename_to=brushpage)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .brush_config
                        .solid_options
                        .pressure_curve = brushpage.solidstyle_pressure_curve();
                }
            ));

        // Chisel style
        imp.chiselstyle_aspect_ratio_row.set_range(1.0, 50.0);
        imp.chiselstyle_aspect_ratio_row.set_value(5.0);
        imp.chiselstyle_fixed_angle_row.set_range(0.0, 360.0);
        imp.chiselstyle_fixed_angle_row.set_value(45.0);

        let update_chisel_profile = clone!(
            #[weak(rename_to=brushpage)]
            self,
            #[weak]
            appwindow,
            move || {
                let profile = brushpage.chisel_tip_profile();
                let mut engine_config = appwindow.engine_config().write();
                engine_config
                    .pens_config
                    .brush_config
                    .chisel_options
                    .tip_profile = profile;
                if matches!(
                    engine_config
                        .pens_config
                        .brush_config
                        .solid_options
                        .tip_profile,
                    TipProfile::Chisel { .. }
                ) {
                    engine_config
                        .pens_config
                        .brush_config
                        .solid_options
                        .tip_profile = profile;
                }
            }
        );

        imp.chiselstyle_aspect_ratio_row.connect_changed(clone!(
            #[strong]
            update_chisel_profile,
            move |_| {
                update_chisel_profile();
            }
        ));

        imp.chiselstyle_fixed_angle_row.connect_changed(clone!(
            #[strong]
            update_chisel_profile,
            move |_| {
                update_chisel_profile();
            }
        ));

        imp.chiselstyle_tilt_responsive_row.connect_active_notify(clone!(
            #[strong]
            update_chisel_profile,
            move |_| {
                update_chisel_profile();
            }
        ));

        // Textured style
        // Density
        imp.texturedstyle_density_row
            .get()
            .set_range(TexturedOptions::DENSITY_MIN, TexturedOptions::DENSITY_MAX);
        // set value after the range!
        imp.texturedstyle_density_row
            .get()
            .set_value(TexturedOptions::default().density);

        imp.texturedstyle_density_row.get().connect_changed(clone!(
            #[weak]
            appwindow,
            move |row| {
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .brush_config
                    .textured_options
                    .density = row.value();
            }
        ));

        // dots distribution
        imp.texturedstyle_distribution_row
            .get()
            .connect_selected_notify(clone!(
                #[weak(rename_to=brushpage)]
                self,
                #[weak]
                appwindow,
                move |_| {
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .brush_config
                        .textured_options
                        .distribution = brushpage.texturedstyle_dots_distribution();
                }
            ));
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        let brush_config = appwindow
            .engine_config()
            .read()
            .pens_config
            .brush_config
            .clone();

        self.set_solidstyle_pressure_curve(brush_config.solid_options.pressure_curve);
        self.set_solidstyle_tip_profile(brush_config.solid_options.tip_profile);
        match brush_config.chisel_options.tip_profile {
            TipProfile::Chisel {
                aspect_ratio,
                fixed_angle,
                use_stylus_tilt,
            } => {
                imp.chiselstyle_aspect_ratio_row.set_value(aspect_ratio);
                imp.chiselstyle_fixed_angle_row
                    .set_value(fixed_angle.to_degrees());
                imp.chiselstyle_tilt_responsive_row
                    .set_active(use_stylus_tilt);
            }
            _ => {}
        }
        imp.texturedstyle_density_row
            .set_value(brush_config.textured_options.density);
        self.set_texturedstyle_distribution_variant(brush_config.textured_options.distribution);

        self.set_brush_style(brush_config.style);
        self.set_buildertype(brush_config.builder_type);

        match brush_config.style {
            BrushStyle::Marker => {
                imp.stroke_width_picker
                    .set_stroke_width(brush_config.marker_options.stroke_width);
            }
            BrushStyle::Solid => {
                imp.stroke_width_picker
                    .set_stroke_width(brush_config.solid_options.stroke_width);
            }
            BrushStyle::Textured => {
                imp.stroke_width_picker
                    .set_stroke_width(brush_config.textured_options.stroke_width);
            }
            BrushStyle::Chisel => {
                imp.stroke_width_picker
                    .set_stroke_width(brush_config.chisel_options.stroke_width);
            }
        }
    }
}
