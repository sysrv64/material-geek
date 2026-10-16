use crate::context::{
    eval_indet_segment, icon_c, lerp, loading_morph_state, mix, r4, text_c, ts, with_alpha, Rect,
    SoftKeyboardMode, UiContext, WidgetId, WidgetResponse, ZoneKind,
};
use crate::props::{
    ButtonProps, CheckState, ChipProps, DatePickerCalendarMode, DatePickerEntryMode,
    DatePickerState, FabSize, IconButtonProps, IconButtonVariant, ProgressProps, PullRefreshState,
    SliderProps, SplitButtonProps, SwipeDismissItem, TextFieldProps, TimePickerState, Variant,
};
use mg_motion::{Tempo, Track};
use mg_render::{
    M3_BODY_LARGE, M3_BODY_MEDIUM, M3_BODY_SMALL, M3_DISPLAY_SMALL, M3_HEADLINE_MEDIUM,
    M3_HEADLINE_SMALL, M3_LABEL_LARGE, M3_LABEL_MEDIUM, M3_LABEL_SMALL, M3_TITLE_LARGE,
    M3_TITLE_MEDIUM, M3_TITLE_SMALL,
};
use std::f32::consts::{FRAC_PI_2, TAU};

impl<'a> UiContext<'a> {
    /// Renders an official Material 3 Expressive Button (Elevated, Filled, Tonal, Outlined, Text) across all 5 size tiers (`Xs`..`Xl`).
    pub fn button(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        props: &ButtonProps,
    ) -> (f32, WidgetResponse) {
        let id = id.into();
        let u = self.u();
        let h = props.size.button_height_dp() * u;
        let pad_h = if props.variant == Variant::Text {
            12.0 * u
        } else {
            props.size.horizontal_padding_dp() * u
        };
        let type_role = match props.size {
            crate::props::Size::Xs => M3_LABEL_MEDIUM,
            crate::props::Size::S => M3_LABEL_LARGE,
            crate::props::Size::M => M3_TITLE_MEDIUM,
            crate::props::Size::L => M3_TITLE_LARGE,
            crate::props::Size::Xl => M3_HEADLINE_SMALL,
        };
        let pal = self.state.pal.clone();
        let ink = if !props.enabled {
            with_alpha(pal.text, 0.38)
        } else {
            match props.variant {
                Variant::Filled => pal.on_primary,
                Variant::Tonal => pal.on_secondary_container,
                Variant::Outlined | Variant::Elevated | Variant::Text => pal.primary,
            }
        };
        let style = ts(type_role, ink, u);
        let tw = self
            .renderer
            .text_width(&props.label, style.px, style.medium, style.tracking_em);
        let icon_px = props.size.icon_dp() * u;
        let icon_w = if props.icon.is_some() {
            icon_px + 8.0 * u
        } else {
            0.0
        };
        let mut w = pad_h + icon_w + tw + pad_h;
        if props.variant != Variant::Text && w < 56.0 * u {
            w = 56.0 * u;
        }
        let rect = Rect::new(x, y, w, h);
        let resp = self.button_rect(id, rect, props);
        (w, resp)
    }

    /// Renders a Material 3 Expressive Button inside an explicit `Rect`.
    pub fn button_rect(
        &mut self,
        id: impl Into<WidgetId>,
        rect: Rect,
        props: &ButtonProps,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let ink = if !props.enabled {
            with_alpha(pal.text, 0.38)
        } else {
            match props.variant {
                Variant::Filled => pal.on_primary,
                Variant::Tonal => pal.on_secondary_container,
                Variant::Outlined | Variant::Elevated | Variant::Text => pal.primary,
            }
        };
        let fill = if !props.enabled {
            match props.variant {
                Variant::Outlined | Variant::Text => None,
                _ => Some(with_alpha(pal.text, 0.12)),
            }
        } else {
            match props.variant {
                Variant::Filled => Some(pal.primary),
                Variant::Tonal => Some(pal.secondary_container),
                Variant::Elevated => Some(pal.card_hi),
                Variant::Outlined | Variant::Text => None,
            }
        };
        let is_p = self.is_pressed(id) && props.enabled;
        let press_t = self
            .state
            .spring_retarget(
                id,
                0,
                if is_p && props.morph_on_press { 1.0 } else { 0.0 },
                Tempo::Fast,
                Track::Spatial,
            )
            .clamp(0.0, 1.0);
        let radius = lerp(rect.h * 0.5, props.size.pressed_radius_dp() * u, press_t);

        if props.variant == Variant::Elevated && props.enabled {
            self.renderer.rect(
                rect.x,
                rect.y + 2.0 * u,
                rect.w,
                rect.h,
                with_alpha(pal.bg, 0.55),
                r4(radius),
            );
        }
        if let Some(f) = fill {
            self.renderer
                .rect(rect.x, rect.y, rect.w, rect.h, f, r4(radius));
        }
        if props.variant == Variant::Outlined {
            let border_col = if props.enabled {
                pal.outline
            } else {
                with_alpha(pal.text, 0.12)
            };
            self.renderer
                .stroke(rect.x, rect.y, rect.w, rect.h, border_col, r4(radius), 1.0 * u);
        }
        if props.enabled {
            self.draw_state_layer(id, rect, r4(radius), ink);
            self.draw_ripples(id, rect);
        }
        let type_role = match props.size {
            crate::props::Size::Xs => M3_LABEL_MEDIUM,
            crate::props::Size::S => M3_LABEL_LARGE,
            crate::props::Size::M => M3_TITLE_MEDIUM,
            crate::props::Size::L => M3_TITLE_LARGE,
            crate::props::Size::Xl => M3_HEADLINE_SMALL,
        };
        let style = ts(type_role, ink, u);
        let tw = self
            .renderer
            .text_width(&props.label, style.px, style.medium, style.tracking_em);
        let icon_px = props.size.icon_dp() * u;
        let icon_w = if props.icon.is_some() {
            icon_px + 8.0 * u
        } else {
            0.0
        };
        let content_w = icon_w + tw;
        let start_x = rect.x + (rect.w - content_w) * 0.5;
        if let Some(ic) = &props.icon {
            icon_c(
                self.renderer,
                ic,
                start_x + icon_px * 0.5,
                rect.y + rect.h * 0.5,
                icon_px,
                ink,
            );
        }
        self.renderer.text(
            &props.label,
            start_x + icon_w,
            rect.y + rect.h * 0.5 - style.line_h * 0.5,
            style,
        );
        if props.enabled {
            self.register_zone(id, rect, ink, ZoneKind::Tap { bounded: true }, false)
        } else {
            WidgetResponse {
                rect,
                ..Default::default()
            }
        }
    }

    /// Renders an M3 Expressive ToggleButton that morphs between pill (20dp) and rounded-square (12dp) on toggle.
    pub fn toggle_button(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        label: &str,
        icon: &str,
        checked: &mut bool,
    ) -> (f32, WidgetResponse) {
        let id = id.into();
        let u = self.u();
        let h = 40.0 * u;
        let anim = self
            .state
            .spring_retarget(id, 0, f32::from(*checked), Tempo::Fast, Track::Spatial)
            .clamp(0.0, 1.0);
        let pal = self.state.pal.clone();
        let fill = mix(pal.card_hi, pal.primary, anim);
        let ink = mix(pal.text, pal.on_primary, anim);
        let style = ts(M3_LABEL_LARGE, ink, u);
        let tw = self
            .renderer
            .text_width(label, style.px, style.medium, style.tracking_em);
        let pad_h = 16.0 * u;
        let icon_w = 24.0 * u;
        let w = pad_h + icon_w + tw + pad_h;
        let rect = Rect::new(x, y, w, h);
        let pressed_f = if self.is_pressed(id) { 1.0 } else { 0.0 };
        let base_r = lerp(20.0 * u, 12.0 * u, anim);
        let radius = lerp(base_r, 8.0 * u, pressed_f);

        self.renderer.rect(x, y, w, h, fill, r4(radius));
        if anim < 0.99 {
            self.renderer.stroke(
                x,
                y,
                w,
                h,
                with_alpha(pal.outline, 1.0 - anim),
                r4(radius),
                1.0 * u,
            );
        }
        self.draw_state_layer(id, rect, r4(radius), ink);
        self.draw_ripples(id, rect);
        icon_c(
            self.renderer,
            if *checked { "check" } else { icon },
            x + pad_h + 8.0 * u,
            y + h * 0.5,
            18.0 * u,
            ink,
        );
        self.renderer
            .text(label, x + pad_h + icon_w, y + h * 0.5 - style.line_h * 0.5, style);
        let mut resp = self.register_zone(id, rect, ink, ZoneKind::Tap { bounded: true }, false);
        if resp.clicked {
            *checked = !*checked;
            self.state
                .spring_retarget(id, 0, f32::from(*checked), Tempo::Fast, Track::Spatial);
            resp.changed = true;
        }
        (w, resp)
    }

    pub fn icon_button(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        icon: &str,
        props: &IconButtonProps,
    ) -> WidgetResponse {
        let u = self.u();
        let w = if props.wide { 52.0 * u } else { 40.0 * u };
        let h = 40.0 * u;
        self.icon_button_sized(id, Rect::new(x, y, w, h), icon, props)
    }

    pub fn icon_button_sized(
        &mut self,
        id: impl Into<WidgetId>,
        rect: Rect,
        icon: &str,
        props: &IconButtonProps,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let (fill, border, ink) = match props.variant {
            IconButtonVariant::Standard => (None, false, pal.text),
            IconButtonVariant::Filled => (Some(pal.primary), false, pal.on_primary),
            IconButtonVariant::Tonal => (
                Some(pal.secondary_container),
                false,
                pal.on_secondary_container,
            ),
            IconButtonVariant::Outlined => (None, true, pal.primary),
        };
        let is_p = self.is_pressed(id) && props.enabled;
        let press_t = self
            .state
            .spring_retarget(id, 0, f32::from(is_p), Tempo::Fast, Track::Spatial)
            .clamp(0.0, 1.0);
        let base_r = rect.w.min(rect.h) * 0.5;
        let radius = lerp(base_r, 8.0 * u, press_t);

        if let Some(f) = fill {
            self.renderer
                .rect(rect.x, rect.y, rect.w, rect.h, f, r4(radius));
        }
        if border {
            self.renderer
                .stroke(rect.x, rect.y, rect.w, rect.h, pal.outline, r4(radius), 1.0 * u);
        }
        self.draw_state_layer(id, rect, r4(radius), ink);
        self.draw_ripples(id, rect);
        icon_c(
            self.renderer,
            icon,
            rect.x + rect.w * 0.5,
            rect.y + rect.h * 0.5,
            22.0 * u,
            ink,
        );
        self.register_zone(id, rect, ink, ZoneKind::Tap { bounded: true }, false)
    }

    pub fn icon_toggle_button(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        icon: &str,
        checked: &mut bool,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let anim = self
            .state
            .spring_retarget(id, 0, f32::from(*checked), Tempo::Fast, Track::Spatial)
            .clamp(0.0, 1.0);
        let pal = self.state.pal.clone();
        let r = lerp(20.0 * u, 12.0 * u, anim);
        let fill = mix(pal.card_hi, pal.primary, anim);
        let ink = mix(pal.text, pal.on_primary, anim);
        let rect = Rect::new(x, y, 40.0 * u, 40.0 * u);
        self.renderer.rect(rect.x, rect.y, rect.w, rect.h, fill, r4(r));
        self.draw_state_layer(id, rect, r4(r), ink);
        self.draw_ripples(id, rect);
        icon_c(
            self.renderer,
            icon,
            rect.x + 20.0 * u,
            rect.y + 20.0 * u,
            22.0 * u,
            ink,
        );
        let mut resp = self.register_zone(id, rect, ink, ZoneKind::Tap { bounded: true }, false);
        if resp.clicked {
            *checked = !*checked;
            self.state
                .spring_retarget(id, 0, f32::from(*checked), Tempo::Fast, Track::Spatial);
            resp.changed = true;
        }
        resp
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fab_styled(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        size: f32,
        radius: f32,
        fill: [f32; 4],
        ink: [f32; 4],
        icon: &str,
        icon_px: f32,
    ) -> (f32, WidgetResponse) {
        let id = id.into();
        let u = self.u();
        let e = self.elev_dp(id) * u;
        let rect = Rect::new(x, y, size, size);
        let shadow_col = with_alpha(self.state.pal.bg, 0.65);
        self.renderer
            .rect(x, y + e, size, size, shadow_col, r4(radius));
        self.renderer.rect(x, y, size, size, fill, r4(radius));
        self.draw_state_layer(id, rect, r4(radius), ink);
        self.draw_ripples(id, rect);
        icon_c(self.renderer, icon, x + size * 0.5, y + size * 0.5, icon_px, ink);
        let resp = self.register_zone(id, rect, ink, ZoneKind::Tap { bounded: true }, false);
        (size, resp)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fab(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        size: FabSize,
        icon: &str,
        fill: [f32; 4],
        ink: [f32; 4],
    ) -> (f32, WidgetResponse) {
        let u = self.u();
        self.fab_styled(
            id,
            x,
            y,
            size.container_dp() * u,
            size.corner_radius_dp() * u,
            fill,
            ink,
            icon,
            size.icon_dp() * u,
        )
    }

    pub fn extended_fab(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        label: &str,
        icon: &str,
        expanded: &mut bool,
    ) -> (f32, WidgetResponse) {
        let id = id.into();
        let u = self.u();
        let anim = self
            .state
            .spring_retarget(id, 0, f32::from(*expanded), Tempo::Default, Track::Spatial)
            .clamp(0.0, 1.0);
        let pal = self.state.pal.clone();
        let ink = pal.on_primary_container;
        let style = ts(M3_LABEL_LARGE, ink, u);
        let tw = self
            .renderer
            .text_width(label, style.px, style.medium, style.tracking_em);
        let full_w = 16.0 * u + 24.0 * u + 12.0 * u + tw + 20.0 * u;
        let fw = lerp(56.0 * u, full_w, anim);
        let e = self.elev_dp(id) * u;
        let rect = Rect::new(x, y, fw, 56.0 * u);

        self.renderer.rect(
            x,
            rect.y + e,
            fw,
            56.0 * u,
            with_alpha(pal.bg, 0.65),
            r4(16.0 * u),
        );
        self.renderer
            .rect(x, rect.y, fw, 56.0 * u, pal.primary_container, r4(16.0 * u));
        self.draw_state_layer(id, rect, r4(16.0 * u), ink);
        self.draw_ripples(id, rect);
        icon_c(self.renderer, icon, x + 28.0 * u, rect.y + 28.0 * u, 24.0 * u, ink);
        if anim > 0.08 {
            self.renderer
                .push_clip([x, rect.y, (fw - 8.0 * u).max(0.0), 56.0 * u]);
            self.renderer.text(
                label,
                x + 52.0 * u,
                rect.y + 28.0 * u - style.line_h * 0.5,
                ts(M3_LABEL_LARGE, with_alpha(ink, anim), u),
            );
            self.renderer.pop_clip();
        }
        let mut resp = self.register_zone(id, rect, ink, ZoneKind::Tap { bounded: true }, false);
        if resp.clicked {
            *expanded = !*expanded;
            self.state
                .spring_retarget(id, 0, f32::from(*expanded), Tempo::Default, Track::Spatial);
            resp.changed = true;
        }
        (fw, resp)
    }

    /// Renders an M3 Expressive `SplitButtonLayout` with interactive popup menu when `props.menu_open` is true.
    pub fn split_button(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        props: &mut SplitButtonProps,
    ) -> (f32, WidgetResponse, Option<usize>) {
        let id = id.into();
        let u = self.u();
        let h = 40.0 * u;
        let pal = self.state.pal.clone();
        let (fill, ink) = if props.tonal {
            (pal.secondary_container, pal.on_secondary_container)
        } else {
            (pal.primary, pal.on_primary)
        };
        let style = ts(M3_LABEL_LARGE, ink, u);
        let ltw =
            self.renderer
                .text_width(&props.main_label, style.px, style.medium, style.tracking_em);
        let lead_w = 16.0 * u + 24.0 * u + ltw + 16.0 * u;
        let lead_id = id.with_index(1);
        let trail_id = id.with_index(2);

        let lead_press = self
            .state
            .spring_retarget(
                lead_id,
                0,
                f32::from(self.is_pressed(lead_id)),
                Tempo::Fast,
                Track::Spatial,
            )
            .clamp(0.0, 1.0);
        let inner_r = lerp(4.0 * u, 12.0 * u, lead_press);
        let lead_radii = [20.0 * u, inner_r, inner_r, 20.0 * u];
        let lead_rect = Rect::new(x, y, lead_w, h);
        self.renderer
            .rect(lead_rect.x, lead_rect.y, lead_rect.w, h, fill, lead_radii);
        self.draw_state_layer(lead_id, lead_rect, lead_radii, ink);
        self.draw_ripples(lead_id, lead_rect);
        if let Some(ic) = &props.leading_icon {
            icon_c(
                self.renderer,
                ic,
                lead_rect.x + 24.0 * u,
                lead_rect.y + h * 0.5,
                18.0 * u,
                ink,
            );
        }
        self.renderer.text(
            &props.main_label,
            lead_rect.x + 38.0 * u,
            lead_rect.y + h * 0.5 - style.line_h * 0.5,
            style,
        );
        let lead_resp =
            self.register_zone(lead_id, lead_rect, ink, ZoneKind::Tap { bounded: true }, false);

        let tx = x + lead_w + 2.0 * u;
        let tw = 44.0 * u;
        let open_anim = self
            .state
            .spring_retarget(
                trail_id,
                0,
                f32::from(props.menu_open),
                Tempo::Fast,
                Track::Spatial,
            )
            .clamp(0.0, 1.0);
        let trail_inner = lerp(4.0 * u, 20.0 * u, open_anim);
        let trail_radii = [trail_inner, 20.0 * u, 20.0 * u, trail_inner];
        let trail_rect = Rect::new(tx, y, tw, h);
        self.renderer
            .rect(tx, trail_rect.y, tw, h, fill, trail_radii);
        self.draw_state_layer(trail_id, trail_rect, trail_radii, ink);
        self.draw_ripples(trail_id, trail_rect);
        let cx = tx + tw * 0.5;
        let cy = trail_rect.y + h * 0.5;
        let dy = lerp(2.4 * u, -2.4 * u, open_anim);
        self.renderer
            .stroke_line([cx - 4.8 * u, cy - dy], [cx, cy + dy], 2.0 * u, ink);
        self.renderer
            .stroke_line([cx, cy + dy], [cx + 4.8 * u, cy - dy], 2.0 * u, ink);
        if self
            .register_zone(
                trail_id,
                trail_rect,
                ink,
                ZoneKind::Tap { bounded: true },
                false,
            )
            .clicked
        {
            props.menu_open = !props.menu_open;
        }

        let mut picked = None;
        if open_anim > 0.04 && !props.menu_items.is_empty() {
            let menu_y = y + h + 6.0 * u;
            let menu_w = (lead_w + tw + 2.0 * u).max(176.0 * u);
            let item_h = 38.0 * u;
            let total_mh = props.menu_items.len() as f32 * item_h + 8.0 * u;
            self.renderer.rect_styled(
                x,
                menu_y,
                menu_w,
                total_mh * open_anim,
                pal.card_highest,
                pal.outline_variant,
                1.0 * u,
                r4(14.0 * u),
                0.0,
            );
            if open_anim > 0.5 {
                for (mi, item_lbl) in props.menu_items.iter().enumerate() {
                    let iy = menu_y + 4.0 * u + mi as f32 * item_h;
                    let irec = Rect::new(x + 4.0 * u, iy, menu_w - 8.0 * u, item_h - 2.0 * u);
                    let iid = id.with_index(10 + mi);
                    let sel = props.selected_item == mi;
                    if sel {
                        self.renderer.rect(
                            irec.x,
                            irec.y,
                            irec.w,
                            irec.h,
                            pal.secondary_container,
                            r4(10.0 * u),
                        );
                    }
                    let i_ink = if sel {
                        pal.on_secondary_container
                    } else {
                        pal.text
                    };
                    self.draw_state_layer(iid, irec, r4(10.0 * u), i_ink);
                    self.draw_ripples(iid, irec);
                    if sel {
                        icon_c(
                            self.renderer,
                            "check",
                            irec.x + 16.0 * u,
                            irec.y + irec.h * 0.5,
                            16.0 * u,
                            i_ink,
                        );
                    }
                    self.renderer.text(
                        item_lbl,
                        irec.x + 30.0 * u,
                        irec.y + 8.0 * u,
                        ts(M3_BODY_MEDIUM, i_ink, u),
                    );
                    if self
                        .register_zone(iid, irec, i_ink, ZoneKind::Tap { bounded: true }, false)
                        .clicked
                    {
                        props.selected_item = mi;
                        props.main_label = item_lbl.clone();
                        props.menu_open = false;
                        picked = Some(mi);
                    }
                }
            }
        }

        (lead_w + 2.0 * u + tw, lead_resp, picked)
    }

    pub fn checkbox(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        ry: f32,
        title: &str,
        state: &mut CheckState,
        tri_state: bool,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let ca = self
            .state
            .spring_retarget(
                id,
                0,
                f32::from(*state != CheckState::Off),
                Tempo::Fast,
                Track::Effects,
            )
            .clamp(0.0, 1.0);
        let cf = self
            .state
            .spring_retarget(
                id,
                1,
                f32::from(*state == CheckState::On),
                Tempo::Default,
                Track::Spatial,
            )
            .clamp(0.0, 1.0);

        let bx = x + 11.0 * u;
        let by = ry + 11.0 * u;
        let hit = Rect::new(x, ry, 40.0 * u, 40.0 * u);
        self.draw_state_circle(id, x + 20.0 * u, ry + 20.0 * u, 20.0 * u, pal.text);
        self.draw_ripples(id, hit);

        if ca > 0.01 {
            self.renderer.rect(
                bx,
                by,
                18.0 * u,
                18.0 * u,
                with_alpha(pal.primary, ca),
                r4(3.5 * u),
            );
        }
        if ca < 0.99 {
            self.renderer.stroke(
                bx,
                by,
                18.0 * u,
                18.0 * u,
                with_alpha(pal.outline, 1.0 - ca),
                r4(3.5 * u),
                2.0 * u,
            );
        }
        if *state == CheckState::Indeterminate {
            self.renderer.rect(
                bx + 4.0 * u,
                by + 8.0 * u,
                10.0 * u,
                2.2 * u,
                with_alpha(pal.on_primary, ca),
                r4(1.1 * u),
            );
        } else if cf > 0.02 {
            let p0 = [bx + 3.8 * u, by + 9.2 * u];
            let p1 = [bx + 7.6 * u, by + 13.0 * u];
            let p2 = [bx + 14.4 * u, by + 5.4 * u];
            let seg1 = (cf / 0.42).clamp(0.0, 1.0);
            let e1 = [lerp(p0[0], p1[0], seg1), lerp(p0[1], p1[1], seg1)];
            self.renderer.stroke_line(p0, e1, 2.2 * u, pal.on_primary);
            if cf > 0.42 {
                let seg2 = ((cf - 0.42) / 0.58).clamp(0.0, 1.0);
                let e2 = [lerp(p1[0], p2[0], seg2), lerp(p1[1], p2[1], seg2)];
                self.renderer.stroke_line(p1, e2, 2.2 * u, pal.on_primary);
            }
        }
        let state_str = match state {
            CheckState::Off => "Unchecked",
            CheckState::On => "Checked",
            CheckState::Indeterminate => "Indeterminate",
        };
        let full_label = format!("{title} • {state_str}");
        self.renderer.text(
            &full_label,
            x + 48.0 * u,
            ry + 8.0 * u,
            ts(M3_BODY_LARGE, pal.text, u),
        );
        let full_hit = Rect::new(x, ry, 290.0 * u, 40.0 * u);
        let mut resp =
            self.register_zone(id, full_hit, pal.text, ZoneKind::Tap { bounded: false }, false);
        if resp.clicked {
            *state = if tri_state {
                state.cycle()
            } else {
                state.toggle_binary()
            };
            resp.changed = true;
        }
        resp
    }

    pub fn switch(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        ry: f32,
        label: &str,
        on: &mut bool,
        with_icons: bool,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let anim = self
            .state
            .spring_retarget(id, 0, f32::from(*on), Tempo::Fast, Track::Spatial)
            .clamp(0.0, 1.0);
        let pressed = self.is_pressed(id);
        let ty = ry + 6.0 * u;
        let cy = ry + 22.0 * u;
        let track_col = mix(pal.card_highest, pal.primary, anim);
        self.renderer
            .rect(x, ty, 52.0 * u, 32.0 * u, track_col, r4(16.0 * u));
        if anim < 0.99 {
            self.renderer.stroke(
                x,
                ty,
                52.0 * u,
                32.0 * u,
                with_alpha(pal.outline, 1.0 - anim),
                r4(16.0 * u),
                2.0 * u,
            );
        }
        let off_size = if with_icons { 24.0 * u } else { 16.0 * u };
        let size = if pressed {
            28.0 * u
        } else {
            lerp(off_size, 24.0 * u, anim)
        };
        let cx = lerp(16.0 * u, 36.0 * u, anim);
        self.draw_state_circle(id, x + cx, cy, 20.0 * u, pal.text);
        self.draw_ripples(id, Rect::new(x, ry, 52.0 * u, 44.0 * u));
        let thumb = mix(pal.outline, pal.on_primary, anim);
        self.renderer.circle(x + cx, cy, size * 0.5, thumb);
        if *on && (with_icons || anim > 0.5) {
            icon_c(self.renderer, "check", x + cx, cy, 16.0 * u, pal.primary);
        } else if !*on && with_icons {
            icon_c(
                self.renderer,
                "close",
                x + cx,
                cy,
                16.0 * u,
                pal.card_highest,
            );
        }
        self.renderer.text(
            label,
            x + 68.0 * u,
            ry + 10.0 * u,
            ts(M3_BODY_LARGE, pal.text, u),
        );
        let mut resp = self.register_zone(
            id,
            Rect::new(x, ry, 280.0 * u, 44.0 * u),
            pal.text,
            ZoneKind::Tap { bounded: false },
            false,
        );
        if resp.clicked {
            *on = !*on;
            resp.changed = true;
        }
        resp
    }

    pub fn radio_button(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        ry: f32,
        label: &str,
        selected: bool,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let cx = x + 20.0 * u;
        let cy = ry + 20.0 * u;
        let ring = if selected { pal.primary } else { pal.outline };
        self.draw_state_circle(id, cx, cy, 20.0 * u, pal.text);
        self.draw_ripples(id, Rect::new(x, ry, 40.0 * u, 40.0 * u));
        self.renderer.stroke(
            cx - 10.0 * u,
            cy - 10.0 * u,
            20.0 * u,
            20.0 * u,
            ring,
            r4(10.0 * u),
            2.0 * u,
        );
        let dot = self
            .state
            .spring_retarget(id, 0, f32::from(selected), Tempo::Fast, Track::Spatial)
            .clamp(0.0, 1.2);
        if dot > 0.02 {
            self.renderer.circle(cx, cy, 5.0 * u * dot, pal.primary);
        }
        self.renderer.text(
            label,
            x + 48.0 * u,
            ry + 8.0 * u,
            ts(M3_BODY_LARGE, pal.text, u),
        );
        self.register_zone(
            id,
            Rect::new(x, ry, 260.0 * u, 40.0 * u),
            pal.text,
            ZoneKind::Tap { bounded: false },
            false,
        )
    }

    /// Renders an M3 Expressive Slider (Standard 16dp track or Wavy GPU sinusoidal track) with live value bubble and optional step ticks.
    #[allow(clippy::too_many_arguments)]
    pub fn slider(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        cy: f32,
        w: f32,
        props: &mut SliderProps,
        wavy: bool,
        leading_icon: Option<&str>,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let hit_rect = Rect::new(x, cy - 22.0 * u, w, 44.0 * u);
        let mut resp = self.register_zone(
            id,
            hit_rect,
            pal.primary,
            ZoneKind::ImmediateHorizontalDrag,
            false,
        );
        let active = resp.dragged || resp.pressed;
        if active && props.enabled {
            let (px, _) = self.pointer_pos();
            let t = ((px - x) / w.max(1.0)).clamp(0.0, 1.0);
            props.set_from_normalized(t);
            resp.changed = true;
        }

        let norm = props.normalized();
        let thumb_anim = self
            .state
            .spring_retarget(id, 0, f32::from(active), Tempo::Fast, Track::Spatial)
            .clamp(0.0, 1.0);
        let tw = lerp(4.0 * u, 2.0 * u, thumb_anim);
        let tx = x + tw * 0.5 + (w - tw) * norm;
        let gap = 6.0 * u;
        let aw = (tx - gap - x).max(0.0);

        if wavy {
            if aw > 2.0 * u {
                self.renderer.wavy_line(
                    x,
                    x + aw,
                    cy,
                    4.0 * u,
                    3.0 * u,
                    28.0 * u,
                    -self.state.t * 5.0,
                    pal.primary,
                );
            }
            let ix = (tx + gap).min(x + w);
            let iw = (x + w - ix).max(0.0);
            if iw > 2.0 * u {
                self.renderer.rect(
                    ix,
                    cy - 2.0 * u,
                    iw,
                    4.0 * u,
                    pal.secondary_container,
                    r4(2.0 * u),
                );
            }
        } else {
            if aw > 2.0 * u {
                self.renderer.rect(
                    x,
                    cy - 8.0 * u,
                    aw,
                    16.0 * u,
                    pal.primary,
                    [8.0 * u, 2.0 * u, 2.0 * u, 8.0 * u],
                );
                if let Some(ic) = leading_icon {
                    if aw > 28.0 * u {
                        icon_c(self.renderer, ic, x + 14.0 * u, cy, 14.0 * u, pal.on_primary);
                    }
                }
            }
            let ix = (tx + gap).min(x + w);
            let iw = (x + w - ix).max(0.0);
            if iw > 2.0 * u {
                self.renderer.rect(
                    ix,
                    cy - 8.0 * u,
                    iw,
                    16.0 * u,
                    pal.secondary_container,
                    [2.0 * u, 8.0 * u, 8.0 * u, 2.0 * u],
                );
            }
        }

        // Step dots if steps > 0
        if props.steps > 0 {
            let intervals = props.steps + 1;
            for s in 0..=intervals {
                let frac = s as f32 / intervals as f32;
                let dx = x + 6.0 * u + (w - 12.0 * u) * frac;
                if (dx - tx).abs() > gap + 2.0 * u {
                    let dot_col = if frac <= norm {
                        pal.on_primary
                    } else {
                        pal.primary
                    };
                    self.renderer.circle(dx, cy, 2.0 * u, dot_col);
                }
            }
        } else {
            let sx = x + w - 6.0 * u;
            if sx > tx + gap + 2.0 * u {
                self.renderer.circle(sx, cy, 2.0 * u, pal.primary);
            }
        }

        let handle_h = if wavy { 40.0 * u } else { 44.0 * u };
        self.renderer.rect(
            tx - tw * 0.5,
            cy - handle_h * 0.5,
            tw,
            handle_h,
            pal.primary,
            r4(tw * 0.5),
        );

        // Value indicator pill above thumb while dragging
        if thumb_anim > 0.08 {
            let pct_lbl = format!("{}%", (norm * 100.0).round() as i32);
            let bw = 40.0 * u * thumb_anim;
            let bh = 24.0 * u * thumb_anim;
            let bx = (tx - bw * 0.5).clamp(x, x + w - bw);
            let by = cy - handle_h * 0.5 - bh - 4.0 * u;
            self.renderer
                .rect(bx, by, bw, bh, pal.inverse, r4(bh * 0.5));
            if thumb_anim > 0.5 {
                text_c(
                    self.renderer,
                    &pct_lbl,
                    bx + bw * 0.5,
                    by + 4.0 * u,
                    ts(M3_LABEL_SMALL, pal.on_inverse, u),
                );
            }
        }
        resp
    }

    /// Renders a two-thumb M3 Expressive `RangeSlider`.
    pub fn range_slider(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        cy: f32,
        w: f32,
        start_val: &mut f32,
        end_val: &mut f32,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let hit_rect = Rect::new(x, cy - 22.0 * u, w, 44.0 * u);
        let mut resp = self.register_zone(
            id,
            hit_rect,
            pal.primary,
            ZoneKind::ImmediateHorizontalDrag,
            false,
        );
        if (resp.dragged || resp.pressed) && w > 1.0 {
            let (px, _) = self.pointer_pos();
            let t = ((px - x) / w).clamp(0.0, 1.0);
            if (t - *start_val).abs() <= (t - *end_val).abs() {
                *start_val = t.min(*end_val - 0.05).max(0.0);
            } else {
                *end_val = t.max(*start_val + 0.05).min(1.0);
            }
            resp.changed = true;
        }
        let s = start_val.clamp(0.0, 1.0);
        let e = end_val.clamp(s + 0.02, 1.0);
        let tx_s = x + w * s;
        let tx_e = x + w * e;
        let gap = 6.0 * u;

        // Left inactive track
        let lw = (tx_s - gap - x).max(0.0);
        if lw > 2.0 * u {
            self.renderer.rect(
                x,
                cy - 6.0 * u,
                lw,
                12.0 * u,
                pal.secondary_container,
                [6.0 * u, 2.0 * u, 2.0 * u, 6.0 * u],
            );
        }
        // Active middle track
        let mx = tx_s + gap;
        let mw = (tx_e - gap - mx).max(0.0);
        if mw > 2.0 * u {
            self.renderer
                .rect(mx, cy - 6.0 * u, mw, 12.0 * u, pal.primary, r4(2.0 * u));
        }
        // Right inactive track
        let rx = (tx_e + gap).min(x + w);
        let rw = (x + w - rx).max(0.0);
        if rw > 2.0 * u {
            self.renderer.rect(
                rx,
                cy - 6.0 * u,
                rw,
                12.0 * u,
                pal.secondary_container,
                [2.0 * u, 6.0 * u, 6.0 * u, 2.0 * u],
            );
        }

        let tw = 4.0 * u;
        self.renderer.rect(
            tx_s - tw * 0.5,
            cy - 18.0 * u,
            tw,
            36.0 * u,
            pal.primary,
            r4(tw * 0.5),
        );
        self.renderer.rect(
            tx_e - tw * 0.5,
            cy - 18.0 * u,
            tw,
            36.0 * u,
            pal.primary,
            r4(tw * 0.5),
        );
        resp
    }

    /// Renders a Material 3 Expressive circular progress indicator (flat or wavy) with front & back track gaps.
    #[allow(clippy::too_many_arguments)]
    pub fn circular_progress(
        &mut self,
        cx: f32,
        cy: f32,
        outer_r: f32,
        stroke_w: f32,
        start_deg: f32,
        sweep_deg: f32,
        wavy: bool,
    ) {
        let u = self.u();
        let pal = self.state.pal.clone();
        let sweep_clamped = sweep_deg.clamp(0.0, 360.0);
        let start_rad = start_deg.to_radians();
        let sweep_rad = sweep_clamped.to_radians();
        let max_amp = if wavy { 1.65 * u } else { 0.0 };
        let base_r = (outer_r - stroke_w * 0.5 - max_amp).max(1.0);
        let gap_rad = if sweep_clamped > 2.0 && sweep_clamped < 356.0 {
            ((4.0 * u + stroke_w) / base_r).min(0.42)
        } else {
            0.0
        };
        let track_sweep = (TAU - sweep_rad - 2.0 * gap_rad).max(0.0);
        if track_sweep > 0.02 {
            let track_outer = base_r + stroke_w * 0.5;
            self.renderer.arc_ring(
                cx,
                cy,
                track_outer,
                stroke_w,
                start_rad + sweep_rad + gap_rad,
                track_sweep,
                pal.secondary_container,
            );
        }
        if sweep_rad > 0.01 {
            if wavy {
                let amp_factor =
                    ((sweep_clamped / 25.0).min((360.0 - sweep_clamped) / 20.0)).clamp(0.35, 1.0);
                let amp = max_amp * amp_factor;
                let phase = -self.state.t * 6.0;
                self.renderer.wavy_arc_ring(
                    cx,
                    cy,
                    base_r + stroke_w * 0.5 + amp,
                    stroke_w,
                    start_rad,
                    sweep_rad,
                    amp,
                    9.0,
                    phase,
                    pal.primary,
                );
            } else {
                self.renderer
                    .arc_ring(cx, cy, outer_r, stroke_w, start_rad, sweep_rad, pal.primary);
            }
        }
    }

    pub fn linear_progress(&mut self, x: f32, cy: f32, w: f32, props: &ProgressProps) {
        let u = self.u();
        let pal = self.state.pal.clone();
        let stroke_w = ProgressProps::STROKE_WIDTH_DP * u;
        let visual_gap = (ProgressProps::TRACK_GAP_DP + ProgressProps::STROKE_WIDTH_DP) * u;
        let phase = -self.state.t * 7.5;

        if let Some(p) = props.progress {
            let p = p.clamp(0.0, 1.0);
            let active_w = w * p;
            if active_w > 1.0 * u {
                if props.wavy {
                    let amp_factor = (p * 10.0).min((1.0 - p) * 10.0).clamp(0.0, 1.0);
                    self.renderer.wavy_line(
                        x,
                        x + active_w,
                        cy,
                        stroke_w,
                        ProgressProps::WAVE_AMPLITUDE_DP * u * amp_factor,
                        ProgressProps::LINEAR_WAVELENGTH_DP * u,
                        phase,
                        pal.primary,
                    );
                } else {
                    self.renderer.rect(
                        x,
                        cy - stroke_w * 0.5,
                        active_w,
                        stroke_w,
                        pal.primary,
                        r4(stroke_w * 0.5),
                    );
                }
            }
            let track_x = (x + active_w + visual_gap).min(x + w);
            let track_w = (x + w - track_x).max(0.0);
            if track_w > 1.0 * u {
                self.renderer.rect(
                    track_x,
                    cy - stroke_w * 0.5,
                    track_w,
                    stroke_w,
                    pal.secondary_container,
                    r4(stroke_w * 0.5),
                );
                self.renderer
                    .circle(x + w - stroke_w * 0.5, cy, stroke_w * 0.5, pal.primary);
            }
        } else {
            let total_ms = 1750.0;
            let t_cycle = ((self.state.t * 1000.0) % total_ms) / total_ms;
            let first_head = eval_indet_segment(t_cycle, 0.0, 1000.0, total_ms);
            let first_tail = eval_indet_segment(t_cycle, 250.0, 1000.0, total_ms);
            let second_head = eval_indet_segment(t_cycle, 650.0, 850.0, total_ms);
            let second_tail = eval_indet_segment(t_cycle, 900.0, 850.0, total_ms);

            let t1_end = x + second_tail * w - visual_gap;
            if t1_end > x + 2.0 * u {
                self.renderer.rect(
                    x,
                    cy - stroke_w * 0.5,
                    t1_end - x,
                    stroke_w,
                    pal.secondary_container,
                    r4(stroke_w * 0.5),
                );
            }
            if second_head > second_tail + 0.01 {
                self.renderer.wavy_line(
                    x + second_tail * w,
                    x + second_head * w,
                    cy,
                    stroke_w,
                    3.0 * u,
                    20.0 * u,
                    phase * 1.15,
                    pal.primary,
                );
            }
            let t2_start = if second_head > 0.0 {
                x + second_head * w + visual_gap
            } else {
                x
            };
            let t2_end = if first_tail < 1.0 {
                x + first_tail * w - visual_gap
            } else {
                x + w
            };
            if t2_end > t2_start + 2.0 * u {
                self.renderer.rect(
                    t2_start,
                    cy - stroke_w * 0.5,
                    t2_end - t2_start,
                    stroke_w,
                    pal.secondary_container,
                    r4(stroke_w * 0.5),
                );
            }
            if first_head > first_tail + 0.01 {
                self.renderer.wavy_line(
                    x + first_tail * w,
                    x + first_head * w,
                    cy,
                    stroke_w,
                    3.0 * u,
                    20.0 * u,
                    phase * 1.15,
                    pal.primary,
                );
            }
            let t3_start = if first_head > 0.0 {
                x + first_head * w + visual_gap
            } else {
                x
            };
            if x + w > t3_start + 2.0 * u {
                self.renderer.rect(
                    t3_start,
                    cy - stroke_w * 0.5,
                    x + w - t3_start,
                    stroke_w,
                    pal.secondary_container,
                    r4(stroke_w * 0.5),
                );
            }
        }
    }

    pub fn chip(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        props: &mut ChipProps,
        toggleable: bool,
    ) -> (f32, WidgetResponse, bool) {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let anim = self
            .state
            .spring_retarget(id, 0, f32::from(props.selected), Tempo::Fast, Track::Spatial)
            .clamp(0.0, 1.0);
        let radius = lerp(8.0 * u, 16.0 * u, anim);
        let ink = if props.selected {
            pal.on_secondary_container
        } else {
            pal.text
        };
        let style = ts(M3_LABEL_LARGE, ink, u);
        let tw = self
            .renderer
            .text_width(&props.label, style.px, style.medium, style.tracking_em);
        let lead_w = if props.leading_icon.is_some() {
            24.0 * u
        } else {
            0.0
        };
        let trail_w = if props.trailing_icon.is_some() {
            24.0 * u
        } else {
            0.0
        };
        let w = 12.0 * u + lead_w + tw + trail_w + 12.0 * u;
        let h = 32.0 * u;
        let rect = Rect::new(x, y, w, h);

        if props.selected {
            self.renderer
                .rect(x, y, w, h, pal.secondary_container, r4(radius));
        } else {
            self.renderer
                .stroke(x, y, w, h, pal.outline, r4(radius), 1.0 * u);
        }
        self.draw_state_layer(id, rect, r4(radius), pal.text);
        self.draw_ripples(id, rect);
        if let Some(ic) = &props.leading_icon {
            icon_c(
                self.renderer,
                ic,
                x + 12.0 * u + 8.0 * u,
                y + h * 0.5,
                18.0 * u,
                if props.selected {
                    pal.on_secondary_container
                } else {
                    pal.primary
                },
            );
        }
        let tx = x + 12.0 * u + lead_w;
        self.renderer
            .text(&props.label, tx, y + h * 0.5 - style.line_h * 0.5, style);

        let mut trailing_clicked = false;
        if let Some(ic) = &props.trailing_icon {
            let tcx = tx + tw + 6.0 * u + 8.0 * u;
            let tcy = y + h * 0.5;
            icon_c(self.renderer, ic, tcx, tcy, 18.0 * u, pal.sub);
            if ic == "close" {
                let trail_id = id.with_index(99);
                let trail_rect = Rect::new(tcx - 12.0 * u, y, 24.0 * u, h);
                if self
                    .register_zone(
                        trail_id,
                        trail_rect,
                        pal.text,
                        ZoneKind::Tap { bounded: false },
                        false,
                    )
                    .clicked
                {
                    trailing_clicked = true;
                }
            }
        }
        let mut resp =
            self.register_zone(id, rect, pal.text, ZoneKind::Tap { bounded: true }, false);
        if resp.clicked && !trailing_clicked && toggleable {
            props.selected = !props.selected;
            resp.changed = true;
        }
        (w, resp, trailing_clicked)
    }

    /// Renders a fully interactive Material 3 Expressive `TextField` / `OutlinedTextField` / `SecureTextField`
    /// with focus state, blinking cursor, physical + M3E soft keyboard input, clear button, password visibility toggle, and validation.
    pub fn text_field(
        &mut self,
        id: impl Into<WidgetId>,
        rect: Rect,
        props: &mut TextFieldProps,
    ) -> WidgetResponse {
        self.text_field_with_kb(id, rect, props, SoftKeyboardMode::Qwerty)
    }

    pub fn text_field_with_kb(
        &mut self,
        id: impl Into<WidgetId>,
        rect: Rect,
        props: &mut TextFieldProps,
        kb_mode: SoftKeyboardMode,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let mut changed_by_keys = self.apply_keys_to_field(id, props);
        let focused = self.is_focused(id);
        let f = self
            .state
            .spring_retarget(id, 0, f32::from(focused), Tempo::Fast, Track::Effects)
            .clamp(0.0, 1.0);
        let has_err = props.error.is_some();
        let accent = if has_err {
            pal.error
        } else {
            mix(pal.sub, pal.primary, f)
        };
        let border_col = if has_err {
            pal.error
        } else {
            mix(pal.outline, pal.primary, f)
        };

        let field_h = rect.h;
        if props.outlined {
            if props.tonal {
                self.renderer.rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    field_h,
                    pal.surface_low,
                    r4(12.0 * u),
                );
            }
            let stroke_w = lerp(1.0 * u, 2.0 * u, f);
            let rad = if props.tonal { 12.0 * u } else { 4.0 * u };
            self.renderer.stroke(
                rect.x,
                rect.y,
                rect.w,
                field_h,
                border_col,
                r4(rad),
                stroke_w,
            );
            let lbl_style = ts(M3_BODY_SMALL, accent, u);
            let ltw = self.renderer.text_width(
                &props.label,
                lbl_style.px,
                lbl_style.medium,
                lbl_style.tracking_em,
            );
            self.renderer.rect(
                rect.x + 12.0 * u,
                rect.y - 4.0 * u,
                ltw + 8.0 * u,
                8.0 * u,
                pal.card,
                r4(0.0),
            );
            self.renderer
                .text(&props.label, rect.x + 16.0 * u, rect.y - 8.0 * u, lbl_style);
        } else {
            let rad = if props.tonal {
                r4(16.0 * u)
            } else {
                [4.0 * u, 4.0 * u, 0.0, 0.0]
            };
            let fill = if props.tonal {
                pal.secondary_container
            } else {
                pal.card_hi
            };
            self.renderer.rect(rect.x, rect.y, rect.w, field_h, fill, rad);
            if !props.tonal {
                let ih = lerp(1.0 * u, 2.5 * u, f);
                self.renderer.rect(
                    rect.x,
                    rect.y + field_h - ih,
                    rect.w,
                    ih,
                    border_col,
                    r4(0.0),
                );
            }
            self.renderer.text(
                &props.label,
                rect.x + if props.leading_icon.is_some() { 48.0 } else { 16.0 } * u,
                rect.y + 7.0 * u,
                ts(M3_BODY_SMALL, accent, u),
            );
        }

        let text_x = rect.x + if props.leading_icon.is_some() { 48.0 } else { 16.0 } * u;
        let text_y = if props.outlined {
            rect.y + field_h * 0.5 - 10.0 * u
        } else {
            rect.y + 25.0 * u
        };

        if let Some(ic) = &props.leading_icon {
            icon_c(
                self.renderer,
                ic,
                rect.x + 24.0 * u,
                rect.y + field_h * 0.5,
                20.0 * u,
                if focused { pal.primary } else { pal.sub },
            );
        }

        // Trailing button (visibility toggle for secure fields, or clear button)
        let mut trail_clicked = false;
        let trail_id = id.with_index(77);
        let trail_rect = Rect::new(
            rect.x + rect.w - 42.0 * u,
            rect.y + field_h * 0.5 - 18.0 * u,
            36.0 * u,
            36.0 * u,
        );
        if props.secure {
            let vis_ic = if props.password_visible {
                "visibility"
            } else {
                "visibility_off"
            };
            self.draw_state_circle(
                trail_id,
                trail_rect.x + 18.0 * u,
                trail_rect.y + 18.0 * u,
                18.0 * u,
                pal.text,
            );
            self.draw_ripples(trail_id, trail_rect);
            icon_c(
                self.renderer,
                vis_ic,
                trail_rect.x + 18.0 * u,
                trail_rect.y + 18.0 * u,
                20.0 * u,
                if props.password_visible {
                    pal.primary
                } else {
                    pal.sub
                },
            );
            if self
                .register_zone(
                    trail_id,
                    trail_rect,
                    pal.text,
                    ZoneKind::Tap { bounded: false },
                    false,
                )
                .clicked
            {
                props.password_visible = !props.password_visible;
                trail_clicked = true;
                changed_by_keys = true;
            }
        } else if props.clearable && !props.text.is_empty() {
            self.draw_state_circle(
                trail_id,
                trail_rect.x + 18.0 * u,
                trail_rect.y + 18.0 * u,
                18.0 * u,
                pal.text,
            );
            self.draw_ripples(trail_id, trail_rect);
            icon_c(
                self.renderer,
                "close",
                trail_rect.x + 18.0 * u,
                trail_rect.y + 18.0 * u,
                18.0 * u,
                pal.sub,
            );
            if self
                .register_zone(
                    trail_id,
                    trail_rect,
                    pal.text,
                    ZoneKind::Tap { bounded: false },
                    false,
                )
                .clicked
            {
                props.clear();
                trail_clicked = true;
                changed_by_keys = true;
            }
        }

        // Prefix / display text / placeholder / suffix + blinking cursor
        let body_style = ts(M3_BODY_LARGE, pal.text, u);
        let max_text_w = (rect.w - (text_x - rect.x) - 44.0 * u).max(20.0 * u);
        self.renderer
            .push_clip([text_x, rect.y + 2.0 * u, max_text_w, field_h - 4.0 * u]);

        let mut cur_x = text_x;
        if let Some(pre) = &props.prefix {
            if !props.text.is_empty() || focused {
                let p_style = ts(M3_BODY_LARGE, pal.sub, u);
                let pw = self.renderer.text(pre, cur_x, text_y, p_style);
                cur_x += pw + 2.0 * u;
            }
        }

        let disp = props.display_text();
        if disp.is_empty() && !focused && !props.placeholder.is_empty() {
            self.renderer.text(
                &props.placeholder,
                cur_x,
                text_y,
                ts(M3_BODY_LARGE, with_alpha(pal.sub, 0.75), u),
            );
        } else if disp.is_empty() && focused && !props.placeholder.is_empty() {
            self.renderer.text(
                &props.placeholder,
                cur_x,
                text_y,
                ts(M3_BODY_LARGE, with_alpha(pal.sub, 0.55), u),
            );
        } else {
            self.renderer.text(&disp, cur_x, text_y, body_style);
        }

        if focused {
            props.clamp_cursor();
            let prefix_chars: String = disp.chars().take(props.cursor).collect();
            let caret_dx = self.renderer.text_width(
                &prefix_chars,
                body_style.px,
                body_style.medium,
                body_style.tracking_em,
            );
            let blink_on = ((self.state.t * 2.2) as u32).is_multiple_of(2);
            if blink_on {
                let caret_col = if has_err { pal.error } else { pal.primary };
                self.renderer.rect(
                    cur_x + caret_dx,
                    text_y + 2.0 * u,
                    2.0 * u,
                    20.0 * u,
                    caret_col,
                    r4(1.0 * u),
                );
            }
        }
        self.renderer.pop_clip();

        // Supporting text / error text and character counter below the field
        let sup_y = rect.y + field_h + 3.0 * u;
        if let Some(err) = &props.error {
            self.renderer.text(
                err,
                rect.x + 12.0 * u,
                sup_y,
                ts(M3_LABEL_SMALL, pal.error, u),
            );
        } else if let Some(sup) = &props.supporting {
            self.renderer.text(
                sup,
                rect.x + 12.0 * u,
                sup_y,
                ts(M3_LABEL_SMALL, pal.sub, u),
            );
        }
        if let Some(max_len) = props.max_length {
            let count_lbl = format!("{}/{}", props.char_len(), max_len);
            let c_style = ts(
                M3_LABEL_SMALL,
                if has_err { pal.error } else { pal.sub },
                u,
            );
            let ctw = self
                .renderer
                .text_width(&count_lbl, c_style.px, c_style.medium, c_style.tracking_em);
            self.renderer
                .text(&count_lbl, rect.x + rect.w - ctw - 8.0 * u, sup_y, c_style);
        }

        let mut resp =
            self.register_zone(id, rect, pal.primary, ZoneKind::Tap { bounded: true }, false);
        if resp.clicked && !trail_clicked && props.enabled {
            self.state.focus(id, kb_mode);
            let (px, _) = self.pointer_pos();
            let rel_x = (px - cur_x).max(0.0);
            let mut best_idx = 0usize;
            let mut best_dist = rel_x;
            let mut acc = String::new();
            for (idx, ch) in disp.chars().enumerate() {
                acc.push(ch);
                let w_ch = self.renderer.text_width(
                    &acc,
                    body_style.px,
                    body_style.medium,
                    body_style.tracking_em,
                );
                let dist = (w_ch - rel_x).abs();
                if dist <= best_dist {
                    best_dist = dist;
                    best_idx = idx + 1;
                }
            }
            props.cursor = best_idx;
            resp.focused = true;
        }
        if changed_by_keys {
            resp.changed = true;
        }
        resp
    }

    /// Renders an authentic Material 3 Expressive `DatePickerDialog` supporting BOTH Calendar mode (Day & Year grids,
    /// month navigation) AND numeric Input mode (toggled by clicking the pencil `"edit"` / `"calendar_today"` button).
    pub fn date_picker(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        avail_w: f32,
        state: &mut DatePickerState,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let dw = (348.0 * u).min(avail_w);
        let dx = x + (avail_w - dw) * 0.5;
        let dh = 440.0 * u;
        let dialog_rect = Rect::new(dx, y, dw, dh);

        self.renderer.rect_styled(
            dx,
            y,
            dw,
            dh,
            pal.card_hi,
            pal.outline_variant,
            1.0 * u,
            r4(28.0 * u),
            0.0,
        );

        // 1. M3EDatePickerHeader (92dp)
        let help_label = match state.entry_mode {
            DatePickerEntryMode::Calendar => "Select date",
            DatePickerEntryMode::Input => "Enter date",
        };
        self.renderer.text(
            help_label,
            dx + 24.0 * u,
            y + 16.0 * u,
            ts(M3_LABEL_MEDIUM, pal.sub, u),
        );
        let headline = state.format_headline();
        self.renderer.text(
            &headline,
            dx + 24.0 * u,
            y + 44.0 * u,
            ts(M3_HEADLINE_MEDIUM, pal.text, u),
        );

        // Pencil ("edit") <-> Calendar ("calendar_today") entry-mode toggle button
        let mode_icon = match state.entry_mode {
            DatePickerEntryMode::Calendar => "edit",
            DatePickerEntryMode::Input => "calendar_today",
        };
        let toggle_btn_id = id.with_index(1);
        let mut changed = false;
        if self
            .icon_button(
                toggle_btn_id,
                dx + dw - 52.0 * u,
                y + 40.0 * u,
                mode_icon,
                &IconButtonProps::standard(),
            )
            .clicked
        {
            state.toggle_entry_mode();
            if state.entry_mode == DatePickerEntryMode::Input {
                self.state
                    .focus(id.with_index(200), SoftKeyboardMode::Numeric);
            } else {
                self.state.clear_focus();
            }
            changed = true;
        }

        // Full-bleed HorizontalDivider below header
        self.renderer
            .rect(dx, y + 92.0 * u, dw, 1.0 * u, pal.outline_variant, r4(0.0));

        match state.entry_mode {
            DatePickerEntryMode::Input => {
                // M3EInputDatePickerFormField mode (numeric date entry in MM/DD/YYYY or YYYY-MM-DD)
                self.renderer.text(
                    "Type date in digits (MM/DD/YYYY or YYYY-MM-DD)",
                    dx + 24.0 * u,
                    y + 112.0 * u,
                    ts(M3_BODY_SMALL, pal.sub, u),
                );
                let field_rect = Rect::new(dx + 24.0 * u, y + 146.0 * u, dw - 48.0 * u, 56.0 * u);
                let field_resp = self.text_field_with_kb(
                    id.with_index(200),
                    field_rect,
                    &mut state.input_field,
                    SoftKeyboardMode::Numeric,
                );
                if field_resp.changed {
                    let _ = state.sync_date_from_input();
                    changed = true;
                }

                // Quick-select numeric date presets for 1-tap testing alongside manual keyboard entry
                self.renderer.text(
                    "Quick date presets:",
                    dx + 24.0 * u,
                    y + 236.0 * u,
                    ts(M3_LABEL_MEDIUM, pal.sub, u),
                );
                let presets = [
                    ("10/16/2026", 2026, 10, 16),
                    ("10/31/2026", 2026, 10, 31),
                    ("12/25/2026", 2026, 12, 25),
                    ("01/01/2027", 2027, 1, 1),
                ];
                let pw = (dw - 48.0 * u - 8.0 * u) * 0.5;
                for (pi, (lbl, py, pm, pd)) in presets.into_iter().enumerate() {
                    let col = pi % 2;
                    let row = pi / 2;
                    let px = dx + 24.0 * u + col as f32 * (pw + 8.0 * u);
                    let pry = y + 260.0 * u + row as f32 * 40.0 * u;
                    let is_cur = state.year == py && state.month == pm && state.day == pd;
                    let mut p_chip = ChipProps::new(lbl, is_cur);
                    p_chip.leading_icon = Some("calendar_today".to_string());
                    let (_, c_resp, _) =
                        self.chip(id.with_index(210 + pi), px, pry, &mut p_chip, false);
                    if c_resp.clicked {
                        state.year = py;
                        state.month = pm;
                        state.day = pd;
                        state.view_year = py;
                        state.view_month = pm;
                        state.sync_input_from_date();
                        changed = true;
                    }
                }
            }
            DatePickerEntryMode::Calendar => {
                // 2. Sub-header: Month/Year mode toggle + Prev/Next month chevrons
                let sub_y = y + 100.0 * u;
                let month_lbl = state.format_sub_header();
                let m_style = ts(M3_LABEL_LARGE, pal.sub, u);
                let m_tw =
                    self.renderer
                        .text_width(&month_lbl, m_style.px, m_style.medium, m_style.tracking_em);
                let mode_rect =
                    Rect::new(dx + 14.0 * u, sub_y + 2.0 * u, m_tw + 38.0 * u, 36.0 * u);
                let mode_id = id.with_index(2);
                self.draw_state_layer(mode_id, mode_rect, r4(18.0 * u), pal.text);
                self.draw_ripples(mode_id, mode_rect);
                self.renderer
                    .text(&month_lbl, dx + 20.0 * u, sub_y + 10.0 * u, m_style);
                icon_c(
                    self.renderer,
                    match state.calendar_mode {
                        DatePickerCalendarMode::Day => "expand_more",
                        DatePickerCalendarMode::Year => "expand_less",
                    },
                    dx + 28.0 * u + m_tw,
                    sub_y + 20.0 * u,
                    18.0 * u,
                    pal.sub,
                );
                if self
                    .register_zone(
                        mode_id,
                        mode_rect,
                        pal.text,
                        ZoneKind::Tap { bounded: true },
                        false,
                    )
                    .clicked
                {
                    state.toggle_calendar_mode();
                    changed = true;
                }

                if state.calendar_mode == DatePickerCalendarMode::Day {
                    if self
                        .icon_button(
                            id.with_index(3),
                            dx + dw - 88.0 * u,
                            sub_y,
                            "chevron_left",
                            &IconButtonProps::standard(),
                        )
                        .clicked
                    {
                        state.prev_month();
                        changed = true;
                    }
                    if self
                        .icon_button(
                            id.with_index(4),
                            dx + dw - 48.0 * u,
                            sub_y,
                            "chevron_right",
                            &IconButtonProps::standard(),
                        )
                        .clicked
                    {
                        state.next_month();
                        changed = true;
                    }

                    // 3. Narrow weekday header row
                    let cw = (dw - 24.0 * u) / 7.0;
                    let x0 = dx + 12.0 * u;
                    let wd = ["S", "M", "T", "W", "T", "F", "S"];
                    for (i, d) in wd.into_iter().enumerate() {
                        text_c(
                            self.renderer,
                            d,
                            x0 + (i as f32 + 0.5) * cw,
                            y + 144.0 * u,
                            ts(M3_BODY_SMALL, pal.sub, u),
                        );
                    }

                    // 4. Day Grid (up to 6 rows x 38dp)
                    let days_count =
                        DatePickerState::days_in_month(state.view_year, state.view_month);
                    let first_wd =
                        DatePickerState::weekday_of(state.view_year, state.view_month, 1);
                    let sel_scale = self
                        .state
                        .spring(id, 10, 1.0, true)
                        .clamp(0.0, 1.15);

                    for day_num in 1..=days_count {
                        let slot = first_wd + (day_num as usize - 1);
                        let col = slot % 7;
                        let row = slot / 7;
                        let gx = x0 + (col as f32 + 0.5) * cw;
                        let gy = y + 170.0 * u + row as f32 * 36.0 * u + 18.0 * u;
                        let sel = state.year == state.view_year
                            && state.month == state.view_month
                            && state.day == day_num;
                        let today = state.view_year == 2026
                            && state.view_month == 10
                            && day_num == 16;
                        let hit = Rect::new(gx - 17.0 * u, gy - 17.0 * u, 34.0 * u, 34.0 * u);
                        let day_id = id.with_index(30 + day_num as usize);
                        if sel {
                            self.renderer
                                .circle(gx, gy, 17.0 * u * sel_scale, pal.primary);
                        } else if today {
                            self.renderer.stroke(
                                hit.x,
                                hit.y,
                                hit.w,
                                hit.h,
                                pal.primary,
                                r4(17.0 * u),
                                1.2 * u,
                            );
                        }
                        self.draw_state_circle(day_id, gx, gy, 17.0 * u, pal.primary);
                        self.draw_ripples(day_id, hit);
                        let label = format!("{day_num}");
                        text_c(
                            self.renderer,
                            &label,
                            gx,
                            gy - 10.0 * u,
                            ts(
                                M3_BODY_MEDIUM,
                                if sel {
                                    pal.on_primary
                                } else if today {
                                    pal.primary
                                } else {
                                    pal.text
                                },
                                u,
                            ),
                        );
                        if self
                            .register_zone(
                                day_id,
                                hit,
                                pal.primary,
                                ZoneKind::Tap { bounded: false },
                                false,
                            )
                            .clicked
                        {
                            state.select_day(day_num);
                            self.state
                                .spring_kick(id, 10, 0.4, 1.0, Tempo::Fast, Track::Spatial);
                            changed = true;
                        }
                    }
                } else {
                    // Year Picker Grid (3 columns x 5 rows: 2020..=2034)
                    let start_year = 2020i32;
                    let col_w = (dw - 32.0 * u) / 3.0;
                    let row_h = 44.0 * u;
                    for idx in 0..15 {
                        let yr = start_year + idx;
                        let col = (idx as usize) % 3;
                        let row = (idx as usize) / 3;
                        let yx = dx + 16.0 * u + col as f32 * col_w + 6.0 * u;
                        let yy = y + 148.0 * u + row as f32 * row_h;
                        let y_rect = Rect::new(yx, yy, col_w - 12.0 * u, 36.0 * u);
                        let y_id = id.with_index(100 + idx as usize);
                        let sel = state.view_year == yr;
                        if sel {
                            self.renderer.rect(
                                y_rect.x,
                                y_rect.y,
                                y_rect.w,
                                y_rect.h,
                                pal.primary,
                                r4(18.0 * u),
                            );
                        }
                        let y_ink = if sel { pal.on_primary } else { pal.text };
                        self.draw_state_layer(y_id, y_rect, r4(18.0 * u), y_ink);
                        self.draw_ripples(y_id, y_rect);
                        text_c(
                            self.renderer,
                            &format!("{yr}"),
                            y_rect.x + y_rect.w * 0.5,
                            y_rect.y + 8.0 * u,
                            ts(M3_BODY_LARGE, y_ink, u),
                        );
                        if self
                            .register_zone(
                                y_id,
                                y_rect,
                                y_ink,
                                ZoneKind::Tap { bounded: true },
                                false,
                            )
                            .clicked
                        {
                            state.select_year(yr);
                            changed = true;
                        }
                    }
                }
            }
        }

        // 5. Dialog Actions Row (status text + Cancel / OK)
        let act_y = y + 388.0 * u;
        if let Some(st_msg) = &state.status_text {
            self.renderer.text(
                st_msg,
                dx + 20.0 * u,
                act_y + 12.0 * u,
                ts(M3_LABEL_SMALL, pal.primary, u),
            );
        }
        let st = ts(M3_LABEL_LARGE, pal.primary, u);
        let wc = 24.0 * u + self.renderer.text_width("Cancel", st.px, st.medium, st.tracking_em);
        let wok = 24.0 * u + self.renderer.text_width("OK", st.px, st.medium, st.tracking_em);
        if self
            .button(
                id.with_index(301),
                dx + dw - wc - wok - 28.0 * u,
                act_y,
                &ButtonProps::text("Cancel"),
            )
            .1
            .clicked
        {
            state.cancel();
            self.state.clear_focus();
            changed = true;
        }
        if self
            .button(
                id.with_index(302),
                dx + dw - wok - 16.0 * u,
                act_y,
                &ButtonProps::text("OK"),
            )
            .1
            .clicked
        {
            if state.confirm() {
                self.state.clear_focus();
            }
            changed = true;
        }

        WidgetResponse {
            clicked: changed,
            changed,
            rect: dialog_rect,
            ..Default::default()
        }
    }

    /// Renders an authentic Material 3 Expressive `TimePickerDialog` supporting both Dial mode (tap & continuous circular drag)
    /// and Input mode (toggled by the bottom-left `"keyboard"` / `"schedule"` icon button).
    pub fn time_picker(
        &mut self,
        id: impl Into<WidgetId>,
        x: f32,
        y: f32,
        avail_w: f32,
        state: &mut TimePickerState,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();
        let dw = (328.0 * u).min(avail_w);
        let dx = x + (avail_w - dw) * 0.5;
        let dh = 432.0 * u;
        let dialog_rect = Rect::new(dx, y, dw, dh);
        let mut changed = false;

        self.renderer.rect_styled(
            dx,
            y,
            dw,
            dh,
            pal.card_hi,
            pal.outline_variant,
            1.0 * u,
            r4(28.0 * u),
            0.0,
        );

        let help = if state.input_mode {
            "Enter time"
        } else {
            "Select time"
        };
        self.renderer.text(
            help,
            dx + 24.0 * u,
            y + 18.0 * u,
            ts(M3_LABEL_MEDIUM, pal.sub, u),
        );

        let fy = y + 44.0 * u;
        let fh = 72.0 * u;
        let fw = 88.0 * u;
        let fields_x = dx + (dw - (fw * 2.0 + 24.0 * u + 12.0 * u + 52.0 * u)) * 0.5;

        if state.input_mode {
            // Input mode: editable Hour and Minute text fields + AM/PM control
            let h_rect = Rect::new(fields_x, fy + 8.0 * u, fw, 56.0 * u);
            if self
                .text_field_with_kb(
                    id.with_index(10),
                    h_rect,
                    &mut state.hour_field,
                    SoftKeyboardMode::Numeric,
                )
                .changed
            {
                let _ = state.sync_time_from_fields();
                changed = true;
            }
            let colon_x = fields_x + fw + 12.0 * u;
            self.renderer.circle(colon_x, fy + 26.0 * u, 3.0 * u, pal.text);
            self.renderer.circle(colon_x, fy + 46.0 * u, 3.0 * u, pal.text);

            let m_rect = Rect::new(fields_x + fw + 24.0 * u, fy + 8.0 * u, fw, 56.0 * u);
            if self
                .text_field_with_kb(
                    id.with_index(11),
                    m_rect,
                    &mut state.minute_field,
                    SoftKeyboardMode::Numeric,
                )
                .changed
            {
                let _ = state.sync_time_from_fields();
                changed = true;
            }

            // Quick time presets in Input mode
            self.renderer.text(
                "Tap Hour or Minute above to type digits, or pick a preset:",
                dx + 24.0 * u,
                y + 150.0 * u,
                ts(M3_BODY_SMALL, pal.sub, u),
            );
            let presets = [
                ("09:00 AM", 9u8, 0u8, true),
                ("12:30 PM", 12, 30, false),
                ("03:15 PM", 3, 15, false),
                ("08:45 PM", 8, 45, false),
            ];
            let pw_chip = (dw - 48.0 * u - 8.0 * u) * 0.5;
            for (pi, (lbl, ph, pm, pam)) in presets.into_iter().enumerate() {
                let col = pi % 2;
                let row = pi / 2;
                let px = dx + 24.0 * u + col as f32 * (pw_chip + 8.0 * u);
                let pry = y + 180.0 * u + row as f32 * 40.0 * u;
                let is_cur = state.hour == ph && state.minute == pm && state.is_am == pam;
                let mut chip = ChipProps::new(lbl, is_cur);
                chip.leading_icon = Some("schedule".to_string());
                if self
                    .chip(id.with_index(40 + pi), px, pry, &mut chip, false)
                    .1
                    .clicked
                {
                    state.hour = ph;
                    state.minute = pm;
                    state.is_am = pam;
                    state.sync_fields_from_time();
                    changed = true;
                }
            }
        } else {
            // Dial mode: Hour and Minute selector boxes
            let h_active = !state.minute_mode;
            let h_rect = Rect::new(fields_x, fy, fw, fh);
            let h_id = id.with_index(1);
            self.renderer.rect_styled(
                h_rect.x,
                h_rect.y,
                h_rect.w,
                h_rect.h,
                if h_active {
                    pal.primary_container
                } else {
                    pal.card_highest
                },
                if h_active { pal.primary } else { [0.0; 4] },
                if h_active { 2.0 * u } else { 0.0 },
                r4(8.0 * u),
                0.0,
            );
            self.draw_state_layer(h_id, h_rect, r4(8.0 * u), pal.text);
            self.draw_ripples(h_id, h_rect);
            let h_str = format!("{:02}", state.hour);
            text_c(
                self.renderer,
                &h_str,
                h_rect.x + fw * 0.5,
                fy + 14.0 * u,
                ts(
                    M3_DISPLAY_SMALL,
                    if h_active {
                        pal.on_primary_container
                    } else {
                        pal.text
                    },
                    u,
                ),
            );
            if self
                .register_zone(h_id, h_rect, pal.text, ZoneKind::Tap { bounded: true }, false)
                .clicked
            {
                state.minute_mode = false;
                changed = true;
            }

            let colon_x = fields_x + fw + 12.0 * u;
            self.renderer.circle(colon_x, fy + 26.0 * u, 3.0 * u, pal.text);
            self.renderer.circle(colon_x, fy + 46.0 * u, 3.0 * u, pal.text);

            let m_active = state.minute_mode;
            let m_rect = Rect::new(fields_x + fw + 24.0 * u, fy, fw, fh);
            let m_id = id.with_index(2);
            self.renderer.rect_styled(
                m_rect.x,
                m_rect.y,
                m_rect.w,
                m_rect.h,
                if m_active {
                    pal.primary_container
                } else {
                    pal.card_highest
                },
                if m_active { pal.primary } else { [0.0; 4] },
                if m_active { 2.0 * u } else { 0.0 },
                r4(8.0 * u),
                0.0,
            );
            self.draw_state_layer(m_id, m_rect, r4(8.0 * u), pal.text);
            self.draw_ripples(m_id, m_rect);
            let m_str = format!("{:02}", state.minute);
            text_c(
                self.renderer,
                &m_str,
                m_rect.x + fw * 0.5,
                fy + 14.0 * u,
                ts(
                    M3_DISPLAY_SMALL,
                    if m_active {
                        pal.on_primary_container
                    } else {
                        pal.text
                    },
                    u,
                ),
            );
            if self
                .register_zone(m_id, m_rect, pal.text, ZoneKind::Tap { bounded: true }, false)
                .clicked
            {
                state.minute_mode = true;
                changed = true;
            }
        }

        // AM / PM Period Control (shared between Dial & Input mode)
        let px = fields_x + fw * 2.0 + 24.0 * u + 12.0 * u;
        let pw = 52.0 * u;
        let ph_half = fh * 0.5;
        let am_rect = Rect::new(px, fy, pw, ph_half);
        let pm_rect = Rect::new(px, fy + ph_half, pw, ph_half);
        if state.is_am {
            self.renderer.rect(
                am_rect.x,
                am_rect.y,
                am_rect.w,
                am_rect.h,
                pal.tertiary_container,
                [8.0 * u, 8.0 * u, 0.0, 0.0],
            );
        } else {
            self.renderer.rect(
                pm_rect.x,
                pm_rect.y,
                pm_rect.w,
                pm_rect.h,
                pal.tertiary_container,
                [0.0, 0.0, 8.0 * u, 8.0 * u],
            );
        }
        self.renderer
            .stroke(px, fy, pw, fh, pal.outline, r4(8.0 * u), 1.0 * u);
        self.renderer
            .rect(px, fy + ph_half - 0.5 * u, pw, 1.0 * u, pal.outline, r4(0.0));

        let am_id = id.with_index(3);
        self.draw_state_layer(am_id, am_rect, [8.0 * u, 8.0 * u, 0.0, 0.0], pal.text);
        self.draw_ripples(am_id, am_rect);
        text_c(
            self.renderer,
            "AM",
            px + pw * 0.5,
            fy + 8.0 * u,
            ts(
                M3_TITLE_SMALL,
                if state.is_am {
                    pal.on_tertiary_container
                } else {
                    pal.sub
                },
                u,
            ),
        );
        if self
            .register_zone(am_id, am_rect, pal.text, ZoneKind::Tap { bounded: true }, false)
            .clicked
        {
            state.is_am = true;
            changed = true;
        }

        let pm_id = id.with_index(4);
        self.draw_state_layer(pm_id, pm_rect, [0.0, 0.0, 8.0 * u, 8.0 * u], pal.text);
        self.draw_ripples(pm_id, pm_rect);
        text_c(
            self.renderer,
            "PM",
            px + pw * 0.5,
            fy + ph_half + 8.0 * u,
            ts(
                M3_TITLE_SMALL,
                if !state.is_am {
                    pal.on_tertiary_container
                } else {
                    pal.sub
                },
                u,
            ),
        );
        if self
            .register_zone(pm_id, pm_rect, pal.text, ZoneKind::Tap { bounded: true }, false)
            .clicked
        {
            state.is_am = false;
            changed = true;
        }

        if !state.input_mode {
            // 3. 224dp Interactive Clock Dial (continuous drag + tap)
            let dial_cx = dx + dw * 0.5;
            let dial_cy = y + 248.0 * u;
            let dial_r = 112.0 * u;
            let knob_r = 20.0 * u;
            let ring_r = dial_r - knob_r - 4.0 * u;
            let dial_id = id.with_index(5);
            let dial_rect = Rect::new(
                dial_cx - dial_r,
                dial_cy - dial_r,
                dial_r * 2.0,
                dial_r * 2.0,
            );
            let dial_resp = self.register_zone(
                dial_id,
                dial_rect,
                pal.primary,
                ZoneKind::DialDrag {
                    cx: dial_cx,
                    cy: dial_cy,
                },
                false,
            );
            if dial_resp.dragged || dial_resp.pressed {
                let (mx, my) = self.pointer_pos();
                state.set_from_dial_point(dial_cx, dial_cy, mx, my);
                changed = true;
            }

            self.renderer.circle(dial_cx, dial_cy, dial_r, pal.card_highest);
            let target_ang = state.dial_angle_rad();
            let hand_ang = self.state.spring_retarget(
                id,
                20,
                target_ang,
                Tempo::Default,
                Track::Spatial,
            );
            let knob_x = dial_cx + ring_r * hand_ang.cos();
            let knob_y = dial_cy + ring_r * hand_ang.sin();
            self.renderer
                .stroke_line([dial_cx, dial_cy], [knob_x, knob_y], 2.2 * u, pal.primary);
            self.renderer.circle(dial_cx, dial_cy, 4.0 * u, pal.primary);
            self.renderer.circle(knob_x, knob_y, knob_r, pal.primary);

            for idx in 0..12 {
                let ang = -FRAC_PI_2 + idx as f32 * (TAU / 12.0);
                let nx = dial_cx + ring_r * ang.cos();
                let ny = dial_cy + ring_r * ang.sin();
                let lbl = if state.minute_mode {
                    format!("{:02}", idx * 5)
                } else if idx == 0 {
                    "12".to_string()
                } else {
                    format!("{idx}")
                };
                let sel = if state.minute_mode {
                    state.minute == (idx as u8) * 5
                } else {
                    (state.hour % 12) == (idx as u8)
                };
                text_c(
                    self.renderer,
                    &lbl,
                    nx,
                    ny - 11.0 * u,
                    ts(
                        M3_BODY_LARGE,
                        if sel { pal.on_primary } else { pal.text },
                        u,
                    ),
                );
            }
        }

        // 4. Bottom Actions Row (Keyboard <-> Dial toggle icon button + status + Cancel / OK)
        let act_y = y + 380.0 * u;
        let toggle_icon = if state.input_mode {
            "schedule"
        } else {
            "keyboard"
        };
        if self
            .icon_button(
                id.with_index(6),
                dx + 12.0 * u,
                act_y,
                toggle_icon,
                &IconButtonProps::standard(),
            )
            .clicked
        {
            state.toggle_input_mode();
            if state.input_mode {
                self.state
                    .focus(id.with_index(10), SoftKeyboardMode::Numeric);
            } else {
                self.state.clear_focus();
            }
            changed = true;
        }
        if let Some(st_msg) = &state.status_text {
            self.renderer.text(
                st_msg,
                dx + 56.0 * u,
                act_y + 12.0 * u,
                ts(M3_LABEL_SMALL, pal.primary, u),
            );
        }
        let st = ts(M3_LABEL_LARGE, pal.primary, u);
        let wc = 24.0 * u + self.renderer.text_width("Cancel", st.px, st.medium, st.tracking_em);
        let wok = 24.0 * u + self.renderer.text_width("OK", st.px, st.medium, st.tracking_em);
        if self
            .button(
                id.with_index(7),
                dx + dw - wc - wok - 28.0 * u,
                act_y,
                &ButtonProps::text("Cancel"),
            )
            .1
            .clicked
        {
            state.cancel();
            self.state.clear_focus();
            changed = true;
        }
        if self
            .button(
                id.with_index(8),
                dx + dw - wok - 16.0 * u,
                act_y,
                &ButtonProps::text("OK"),
            )
            .1
            .clicked
        {
            if state.confirm() {
                self.state.clear_focus();
            }
            changed = true;
        }

        WidgetResponse {
            clicked: changed,
            changed,
            rect: dialog_rect,
            ..Default::default()
        }
    }

    /// Renders an interactive Material 3 Expressive `SwipeToDismissBox` item with full horizontal drag physics,
    /// dynamic background pill reveal, threshold detection, spring snap-back/fly-out, and Undo restoration.
    pub fn swipe_to_dismiss(
        &mut self,
        id: impl Into<WidgetId>,
        rect: Rect,
        item: &mut SwipeDismissItem,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();

        if item.dismissed {
            // Dismissed banner with Undo button
            let bg_col = if item.dismissed_start_to_end {
                pal.secondary_container
            } else {
                pal.error_container
            };
            let ink_col = if item.dismissed_start_to_end {
                pal.on_secondary_container
            } else {
                pal.on_error_container
            };
            self.renderer
                .rect(rect.x, rect.y, rect.w, rect.h, bg_col, r4(16.0 * u));
            let msg = if item.dismissed_start_to_end {
                format!("Archived \"{}\"", item.headline)
            } else {
                format!("Deleted \"{}\"", item.headline)
            };
            self.renderer.text(
                &msg,
                rect.x + 16.0 * u,
                rect.y + rect.h * 0.5 - 10.0 * u,
                ts(M3_BODY_MEDIUM, ink_col, u),
            );
            let undo_id = id.with_index(99);
            let undo_rect = Rect::new(
                rect.x + rect.w - 84.0 * u,
                rect.y + rect.h * 0.5 - 16.0 * u,
                72.0 * u,
                32.0 * u,
            );
            self.renderer.rect(
                undo_rect.x,
                undo_rect.y,
                undo_rect.w,
                undo_rect.h,
                with_alpha(ink_col, 0.16),
                r4(16.0 * u),
            );
            self.draw_state_layer(undo_id, undo_rect, r4(16.0 * u), ink_col);
            self.draw_ripples(undo_id, undo_rect);
            text_c(
                self.renderer,
                "Undo",
                undo_rect.x + undo_rect.w * 0.5,
                undo_rect.y + 7.0 * u,
                ts(M3_LABEL_LARGE, ink_col, u),
            );
            let mut resp = self.register_zone(
                undo_id,
                undo_rect,
                ink_col,
                ZoneKind::Tap { bounded: true },
                false,
            );
            if resp.clicked {
                item.reset();
                self.state
                    .spring_kick(id, 0, 0.0, 0.0, Tempo::Fast, Track::Spatial);
                resp.changed = true;
            }
            return resp;
        }

        let mut resp = self.register_zone(
            id,
            rect,
            pal.text,
            ZoneKind::SwipeHorizontalDrag,
            false,
        );
        let threshold_dp = 88.0;
        if resp.dragged {
            item.offset_dp = (item.offset_dp + resp.drag_delta_x / u).clamp(-260.0, 260.0);
            item.target_dp = item.offset_dp;
            self.state.spring_kick(
                id,
                0,
                item.offset_dp,
                item.offset_dp,
                Tempo::Fast,
                Track::Spatial,
            );
            resp.changed = true;
        } else {
            if item.offset_dp.abs() >= threshold_dp {
                item.dismissed = true;
                item.dismissed_start_to_end = item.offset_dp > 0.0;
                item.offset_dp = 0.0;
                item.target_dp = 0.0;
                resp.changed = true;
                return resp;
            }
            item.target_dp = 0.0;
            item.offset_dp = self.state.spring_retarget(
                id,
                0,
                0.0,
                Tempo::Default,
                Track::Spatial,
            );
        }

        let offset_px = item.offset_dp * u;
        let abs_px = offset_px.abs();
        let progress = (item.offset_dp.abs() / threshold_dp).clamp(0.0, 1.0);

        // Background action reveal (M3E pill background + scaling icon & label)
        self.renderer
            .rect(rect.x, rect.y, rect.w, rect.h, pal.surface_low, r4(16.0 * u));
        if abs_px > 4.0 * u {
            let swiping_right = offset_px > 0.0;
            let bg_fill = if swiping_right {
                mix(pal.secondary_container, pal.primary, progress)
            } else {
                mix(pal.error_container, pal.error, progress * 0.45)
            };
            let bg_ink = if swiping_right {
                mix(pal.on_secondary_container, pal.on_primary, progress)
            } else {
                pal.on_error_container
            };
            let pill_w = abs_px.min(rect.w);
            let pill_x = if swiping_right {
                rect.x
            } else {
                rect.x + rect.w - pill_w
            };
            let pill_r = lerp(16.0 * u, rect.h * 0.5, progress);
            self.renderer
                .rect(pill_x, rect.y, pill_w, rect.h, bg_fill, r4(pill_r));
            let ic_scale = lerp(0.8, 1.15, progress);
            let ic_px = 20.0 * u * ic_scale;
            if swiping_right {
                icon_c(
                    self.renderer,
                    "archive",
                    rect.x + 26.0 * u,
                    rect.y + rect.h * 0.5,
                    ic_px,
                    bg_ink,
                );
                if pill_w > 84.0 * u {
                    self.renderer.text(
                        "Archive",
                        rect.x + 48.0 * u,
                        rect.y + rect.h * 0.5 - 10.0 * u,
                        ts(M3_LABEL_LARGE, bg_ink, u),
                    );
                }
            } else {
                icon_c(
                    self.renderer,
                    "delete",
                    rect.x + rect.w - 26.0 * u,
                    rect.y + rect.h * 0.5,
                    ic_px,
                    bg_ink,
                );
                if pill_w > 84.0 * u {
                    self.renderer.text(
                        "Delete",
                        rect.x + rect.w - 92.0 * u,
                        rect.y + rect.h * 0.5 - 10.0 * u,
                        ts(M3_LABEL_LARGE, bg_ink, u),
                    );
                }
            }
        }

        // Foreground draggable card clipped to row bounds
        self.renderer
            .push_clip([rect.x, rect.y, rect.w, rect.h]);
        let fg_x = rect.x + offset_px;
        let fg_r = lerp(16.0 * u, 24.0 * u, progress);
        self.renderer
            .rect(fg_x, rect.y, rect.w, rect.h, pal.card_hi, r4(fg_r));
        self.draw_state_layer(
            id,
            Rect::new(fg_x, rect.y, rect.w, rect.h),
            r4(fg_r),
            pal.text,
        );
        self.renderer.m3_shape(
            fg_x + 28.0 * u,
            rect.y + rect.h * 0.5,
            34.0 * u,
            item.shape,
            0.0,
            pal.primary_container,
            [0.0; 4],
            0.0,
        );
        icon_c(
            self.renderer,
            &item.icon,
            fg_x + 28.0 * u,
            rect.y + rect.h * 0.5,
            17.0 * u,
            pal.on_primary_container,
        );
        self.renderer.text(
            &item.headline,
            fg_x + 54.0 * u,
            rect.y + 10.0 * u,
            ts(M3_BODY_LARGE, pal.text, u),
        );
        self.renderer.text(
            &item.supporting,
            fg_x + 54.0 * u,
            rect.y + 32.0 * u,
            ts(M3_BODY_SMALL, pal.sub, u),
        );
        let tr_style = ts(M3_LABEL_SMALL, pal.sub, u);
        let tr_w =
            self.renderer
                .text_width(&item.trailing, tr_style.px, tr_style.medium, tr_style.tracking_em);
        self.renderer.text(
            &item.trailing,
            fg_x + rect.w - 16.0 * u - tr_w,
            rect.y + 14.0 * u,
            tr_style,
        );
        self.renderer.pop_clip();

        resp
    }

    /// Renders an interactive Material 3 Expressive `PullToRefreshBox` with vertical drag-to-pull physics and timed refresh cycle.
    pub fn pull_to_refresh(
        &mut self,
        id: impl Into<WidgetId>,
        rect: Rect,
        state: &mut PullRefreshState,
    ) -> WidgetResponse {
        let id = id.into();
        let u = self.u();
        let pal = self.state.pal.clone();

        if state.refreshing {
            if let Some(t0) = state.refresh_started_at {
                if self.state.t - t0 >= 1.8 {
                    state.refreshing = false;
                    state.refresh_started_at = None;
                    state.pull_fraction = 0.0;
                    state.refresh_count += 1;
                }
            } else {
                state.refresh_started_at = Some(self.state.t);
            }
        }

        let mut resp =
            self.register_zone(id, rect, pal.primary, ZoneKind::VerticalPullDrag, false);
        if resp.dragged && !state.refreshing {
            state.pull_fraction =
                (state.pull_fraction + resp.drag_delta_y / (80.0 * u)).clamp(0.0, 1.35);
            resp.changed = true;
        } else if !state.refreshing {
            if state.pull_fraction >= 0.85 {
                state.refreshing = true;
                state.refresh_started_at = Some(self.state.t);
                state.pull_fraction = 1.0;
                resp.changed = true;
            } else {
                state.pull_fraction = self
                    .state
                    .spring_retarget(id, 0, 0.0, Tempo::Default, Track::Spatial)
                    .clamp(0.0, 1.2);
            }
        }

        self.renderer
            .rect(rect.x, rect.y, rect.w, rect.h, pal.surface_low, r4(16.0 * u));
        let cx = rect.x + 34.0 * u;
        let cy = rect.y + rect.h * 0.5;
        let vis_scale = if state.refreshing {
            1.0
        } else {
            state.pull_fraction.clamp(0.45, 1.1)
        };
        let r_cont = 22.0 * u * vis_scale;
        self.renderer
            .circle(cx, cy + 2.0 * u, r_cont, with_alpha(pal.bg, 0.55));
        self.renderer.circle(cx, cy, r_cont, pal.card_highest);
        let anim_t = if state.refreshing {
            self.state.t
        } else {
            state.pull_fraction * 2.5
        };
        let (sa, sb, mt, rot, sc) = loading_morph_state(anim_t, 1);
        self.renderer.m3_shape_morph(
            cx,
            cy,
            30.0 * u * sc * vis_scale,
            sa,
            sb,
            mt,
            rot,
            pal.primary,
            [0.0; 4],
            0.0,
        );

        let title = if state.refreshing {
            "Refreshing feed... (7-shape morph)"
        } else if state.pull_fraction > 0.1 {
            "Release to trigger refresh"
        } else {
            "Pull down or tap Refresh"
        };
        self.renderer.text(
            title,
            rect.x + 68.0 * u,
            rect.y + 14.0 * u,
            ts(M3_TITLE_SMALL, pal.text, u),
        );
        let sub = format!("Feed sync #{} • ContainedLoadingIndicator", state.refresh_count);
        self.renderer.text(
            &sub,
            rect.x + 68.0 * u,
            rect.y + 36.0 * u,
            ts(M3_BODY_SMALL, pal.sub, u),
        );

        let btn_w = 88.0 * u;
        let btn_rect = Rect::new(
            rect.x + rect.w - btn_w - 12.0 * u,
            rect.y + rect.h * 0.5 - 16.0 * u,
            btn_w,
            32.0 * u,
        );
        if self
            .button_rect(
                id.with_index(1),
                btn_rect,
                &ButtonProps::tonal(if state.refreshing { "Syncing" } else { "Refresh" })
                    .with_size(crate::props::Size::Xs),
            )
            .clicked
            && !state.refreshing
        {
            state.refreshing = true;
            state.refresh_started_at = Some(self.state.t);
            state.pull_fraction = 1.0;
            resp.changed = true;
        }
        resp
    }
}
