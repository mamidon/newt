use drawing::typesetting::{GlyphRun, TypeSet};
use drawing::{Brush, DrawList, Extent, ShapeKind};
use euclid::default::Vector2D;
use euclid::Point2D;
use euclid::Rect;
use euclid::Size2D;
use std::borrow::Borrow;
use std::cmp::{max, min};
use std::ops::Add;
use std::rc::Rc;

pub struct Pixels;
pub type Position = Point2D<i64, Pixels>;
pub type Dimensions = Size2D<i64, Pixels>;
pub type Rectangle = Rect<i64, Pixels>;

#[derive(Copy, Clone)]
pub struct LayoutSpace {
    width: Option<u32>,
    height: Option<u32>,
}

impl LayoutSpace {
    pub fn window(width: u32, height: u32) -> LayoutSpace {
        LayoutSpace {
            width: Some(width),
            height: Some(height),
        }
    }

    pub fn with_height(&self, height: Option<u32>) -> LayoutSpace {
        LayoutSpace {
            width: self.width,
            height,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RenderSpace {
    pub position: Position,
    pub dimensions: Dimensions,
}

impl RenderSpace {
    pub fn new(position: Position, dimensions: Dimensions) -> RenderSpace {
        RenderSpace {
            position,
            dimensions,
        }
    }

    fn center_horizontally(&self, inner: &RenderSpace) -> RenderSpace {
        let outer_width = self.dimensions.width;
        let inner_width = inner.dimensions.width;

        let position = if outer_width >= inner_width {
            Position::new((outer_width - inner_width) / 2, self.position.y)
        } else {
            panic!("Cannot horizontally center a larger RenderSpace inside a smaller RenderSpace")
        };

        RenderSpace {
            position,
            dimensions: inner.dimensions,
        }
    }

    fn subsect_vertically(&self, inner: &RenderSpace) -> RenderSpace {
        let outer_height = self.dimensions.height;
        let inner_height = inner.dimensions.height;

        if outer_height < inner_height {
            panic!("Cannot vertically subset a smaller RenderSpace by a larger RenderSpace");
        }

        let position = Position::new(self.position.x, self.position.y + inner_height);
        let dimensions = Dimensions::new(self.dimensions.width, outer_height - inner_height);

        RenderSpace {
            position,
            dimensions,
        }
    }

    fn from_dimensions(dimensions: Dimensions) -> RenderSpace {
        RenderSpace::new(Position::default(), dimensions)
    }

    fn as_layout_space(&self) -> LayoutSpace {
        LayoutSpace {
            width: Some(self.dimensions.width as u32),
            height: Some(self.dimensions.height as u32),
        }
    }
}

pub struct LayoutNode {
    pub item: Rc<LayoutItem>,
    pub children: Vec<LayoutNode>,
}

pub struct RenderNode {
    pub item: Rc<LayoutItem>,
    pub children: Vec<RenderNode>,
    pub render_space: RenderSpace,
}

pub struct RenderNodeIterator<'a> {
    frontier: Vec<&'a RenderNode>,
}

pub enum LayoutItem {
    Box {
        width: u32,
        height: u32,
    },
    Stack,
    Shape {
        kind: ShapeKind,
        brush: Brush,
        dimensions: Dimensions,
    },
    Text {
        lines: Vec<GlyphRun>,
        type_set: TypeSet,
    },
}

impl LayoutNode {
    pub fn new_box(width: u32, height: u32, children: Vec<LayoutNode>) -> LayoutNode {
        LayoutNode {
            item: Rc::new(LayoutItem::Box { width, height }),
            children,
        }
    }

    pub fn new_stack(children: Vec<LayoutNode>) -> LayoutNode {
        LayoutNode {
            item: Rc::new(LayoutItem::Stack),
            children,
        }
    }

    pub fn new_shape(kind: ShapeKind, brush: Brush, dimensions: Dimensions) -> LayoutNode {
        LayoutNode {
            item: Rc::new(LayoutItem::Shape {
                kind,
                brush,
                dimensions,
            }),
            children: Vec::new(),
        }
    }

    pub fn new_text(text: &str, type_set: TypeSet) -> LayoutNode {
        let mut lines: Vec<GlyphRun> = Vec::new();

        for line in text.split('\n') {
            let mut glyph_run = type_set.build_glyph_run();
            let glyphs = type_set.as_typeset_glyphs(line);

            glyph_run.append(glyphs.as_slice());

            lines.push(glyph_run);
        }

        LayoutNode {
            item: Rc::new(LayoutItem::Text { lines, type_set }),
            children: Vec::new(),
        }
    }

    pub fn layout(&self, mut layout_space: LayoutSpace) -> RenderNode {
        match self.item.borrow() {
            LayoutItem::Box { width, height } => {
                LayoutNode::layout_box(*width, *height, &self.children)
            }
            LayoutItem::Shape { dimensions, .. } => {
                LayoutNode::layout_shape(self.item.clone(), *dimensions)
            }
            LayoutItem::Stack {} => {
                LayoutNode::layout_stack(self.item.clone(), &layout_space, &self.children)
            }
            LayoutItem::Text { lines, type_set } => LayoutNode::layout_text(lines, type_set),
        }
    }

    fn layout_box(width: u32, height: u32, children: &Vec<LayoutNode>) -> RenderNode {
        unimplemented!()
    }

    fn layout_shape(layout_item: Rc<LayoutItem>, dimensions: Dimensions) -> RenderNode {
        RenderNode::leaf(layout_item, RenderSpace::from_dimensions(dimensions))
    }

    fn layout_stack(
        layout_item: Rc<LayoutItem>,
        layout_space: &LayoutSpace,
        children: &Vec<LayoutNode>,
    ) -> RenderNode {
        let unspecified_height = layout_space.with_height(None);

        let mut render_children: Vec<RenderNode> = children
            .iter()
            .map(|c| c.layout(unspecified_height))
            .collect();

        let dimensions = {
            let render_children_width = render_children
                .iter()
                .map(|c| c.render_space.dimensions.width)
                .max()
                .unwrap_or(0);

            let render_children_height: i64 = render_children
                .iter()
                .map(|c| c.render_space.dimensions.height)
                .sum();

            let required_width: i64 =
                layout_space.width.unwrap_or(render_children_width as u32) as i64;
            let required_height: i64 =
                layout_space.height.unwrap_or(render_children_height as u32) as i64;

            Dimensions::new(required_width, required_height)
        };

        let container_render_space = RenderSpace::from_dimensions(dimensions);

        let mut remaining_render_space = container_render_space;

        for child in &mut render_children {
            child.render_space = remaining_render_space.center_horizontally(&child.render_space);
            remaining_render_space = remaining_render_space.subsect_vertically(&child.render_space);
        }

        RenderNode::container(layout_item, container_render_space, render_children)
    }

    fn layout_text(lines: &Vec<GlyphRun>, type_set: &TypeSet) -> RenderNode {
        unimplemented!()
    }
}

impl RenderNode {
    pub fn leaf(item: Rc<LayoutItem>, render_space: RenderSpace) -> RenderNode {
        RenderNode {
            item,
            render_space,
            children: Vec::new(),
        }
    }

    pub fn container(
        item: Rc<LayoutItem>,
        render_space: RenderSpace,
        children: Vec<RenderNode>,
    ) -> RenderNode {
        RenderNode {
            item,
            render_space,
            children,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &RenderNode> {
        RenderNodeIterator {
            frontier: vec![self],
        }
    }
}

impl<'a> Iterator for RenderNodeIterator<'a> {
    type Item = &'a RenderNode;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.frontier.pop();

        if let Some(node) = next {
            for child in node.children.iter().rev() {
                self.frontier.push(child);
            }
        }

        next
    }
}

mod tests {
    use crate::layout::{Dimensions, LayoutNode, LayoutSpace, Position, RenderNode, RenderSpace};
    use drawing::{Brush, ShapeKind};

    #[test]
    fn stacks_playground() {
        let layout_space = LayoutSpace {
            width: Some(100),
            height: None,
        };

        let mut child_render_spaces = vec![
            RenderSpace::from_dimensions(Dimensions::new(25, 5)),
            RenderSpace::from_dimensions(Dimensions::new(25, 5)),
            RenderSpace::from_dimensions(Dimensions::new(25, 5)),
            RenderSpace::from_dimensions(Dimensions::new(25, 5)),
            RenderSpace::from_dimensions(Dimensions::new(25, 5)),
        ];

        let height_of_children: i64 = child_render_spaces
            .iter()
            .map(|c| c.dimensions.height)
            .sum();
        let container_render_space = RenderSpace::from_dimensions(Dimensions::new(
            layout_space.width.unwrap_or(
                (child_render_spaces
                    .iter()
                    .map(|c| c.dimensions.width)
                    .max()
                    .unwrap()
                    * 3) as u32,
            ) as i64,
            layout_space.height.unwrap_or(height_of_children as u32) as i64,
        ));

        let mut remaining_render_space = container_render_space;
        for child_render_space in &mut child_render_spaces {
            *child_render_space = remaining_render_space.center_horizontally(child_render_space);
            remaining_render_space = remaining_render_space.subsect_vertically(&child_render_space);
        }

        println!("Container RenderSpace: {:?}", container_render_space);
        for (index, child_render_space) in child_render_spaces.iter().enumerate() {
            println!(
                "\tChild {}/{} RenderSpace: {:?}",
                index,
                child_render_spaces.len(),
                child_render_space
            );
        }
    }

    #[test]
    fn layoutnode_can_traverse_children_after_layout() {
        let brush = Brush {
            foreground: 0xFF0000FF,
            background: 0x00FF00FF,
        };
        let dimensions = Dimensions::new(150, 150);

        let layout_root = LayoutNode::new_stack(vec![
            LayoutNode::new_shape(ShapeKind::Rectangle, brush, dimensions),
            LayoutNode::new_shape(ShapeKind::Rectangle, brush, dimensions),
            LayoutNode::new_shape(ShapeKind::Ellipse, brush, dimensions),
        ]);
        let render_root = layout_root.layout(LayoutSpace::window(1024, 1024));
        let render_nodes: Vec<&RenderNode> = render_root.iter().collect();
        let positions: Vec<Position> = render_nodes
            .iter()
            .map(|n| n.render_space.position)
            .collect();
        let dimensions: Vec<Dimensions> = render_nodes
            .iter()
            .map(|n| n.render_space.dimensions)
            .collect();

        assert_eq!(4, render_nodes.len());

        assert_eq!(Some(&Position::new(0, 0)), positions.get(0));
        assert_eq!(Some(&Dimensions::new(1024, 1024)), dimensions.get(0));

        assert_eq!(Some(&Position::new(437, 0)), positions.get(1));
        assert_eq!(Some(&Dimensions::new(150, 150)), dimensions.get(1));

        assert_eq!(Some(&Position::new(437, 150)), positions.get(2));
        assert_eq!(Some(&Dimensions::new(150, 150)), dimensions.get(2));

        assert_eq!(Some(&Position::new(437, 300)), positions.get(3));
        assert_eq!(Some(&Dimensions::new(150, 150)), dimensions.get(3));
    }
}
