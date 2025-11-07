use crate::dom::{self, Pattern, Rect, Svg, node::DomNode};

#[derive(Clone, Debug, PartialEq)]
pub struct GridSettings {
    pub enabled: bool,
    pub snap: bool,
    pub size: f64,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap: true,
            size: 20.0,
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    pub grid_svg: DomNode<Svg>,
    pattern: DomNode<Pattern>,
    rect: DomNode<Rect>,
}

impl Grid {
    pub fn new(mut grid_svg: DomNode<Svg>) -> Self {
        grid_svg.remove_on_drop();

        let defs = dom::defs().mount(&grid_svg);

        let pattern = dom::pattern()
            .id("penguin-dot-grid")
            .x(0.)
            .y(0.)
            .pattern_units("userSpaceOnUse")
            .mount(&defs);

        dom::circle()
            .cx(0.)
            .cy(0.)
            .r(1.5)
            .fill("rgba(255,255,255,0.15)")
            .mount(&pattern);

        let rect = dom::rect()
            .x(-10000.)
            .y(-10000.)
            .width(20000.)
            .height(20000.)
            .fill("url(#penguin-dot-grid)")
            .mount(&grid_svg);

        Self {
            grid_svg,
            pattern,
            rect,
        }
    }

    pub fn update_grid_settings(&self, gs: &GridSettings) {
        if gs.enabled {
            self.rect.show();
        } else {
            self.rect.hide();
        }

        self.pattern.set_svg_width(gs.size);
        self.pattern.set_svg_height(gs.size);
    }
}
