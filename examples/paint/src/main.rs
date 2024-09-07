#![allow(dead_code, unused_imports)]
use iced::{
    alignment::{Horizontal, Vertical},
    color,
    theme::palette,
    widget::{
        button, column, container, row, text, tooltip, vertical_rule,
        vertical_slider, vertical_space, Column, Container,
    },
    Color, Element, Font, Length, Theme,
};

use canvas::{Painting, State};

const ICON_FONT: Font = Font::with_name("paint-icons");

fn main() -> iced::Result {
    iced::application("Iced Paint", Paint::update, Paint::view)
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .font(include_bytes!("../fonts/paint-icons.ttf").as_slice())
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PaintColor {
    Black(f32),
    White,
    Grey,
    Ivory,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
    Rose,
    Cyan,
    Fuchsia,
    Empty,
    Custom(Color),
}

impl PaintColor {
    const ALL: [PaintColor; 14] = [
        Self::White,
        Self::Black(1.0),
        Self::Grey,
        Self::Ivory,
        Self::Red,
        Self::Orange,
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::Indigo,
        Self::Violet,
        Self::Fuchsia,
        Self::Rose,
        Self::Cyan,
    ];

    fn opacity(&mut self, opacity: f32) -> Self {
        match self {
            Self::Black(_) => Self::Black(opacity),
            _ => *self,
        }
    }
}

impl Default for PaintColor {
    fn default() -> Self {
        Self::Black(1.0)
    }
}

impl From<PaintColor> for Color {
    fn from(value: PaintColor) -> Self {
        match value {
            PaintColor::Black(alpha) => color!(0, 0, 0, alpha),
            PaintColor::White => color!(255, 255, 255),
            PaintColor::Grey => color!(71, 85, 105),
            PaintColor::Ivory => color!(240, 234, 214),
            PaintColor::Red => color!(255, 0, 0),
            PaintColor::Green => color!(0, 255, 0),
            PaintColor::Blue => color!(0, 0, 255),
            PaintColor::Orange => color!(234, 88, 12),
            PaintColor::Yellow => color!(234, 179, 8),
            PaintColor::Indigo => color!(79, 70, 229),
            PaintColor::Violet => color!(124, 58, 237),
            PaintColor::Rose => color!(225, 29, 72),
            PaintColor::Cyan => color!(8, 145, 178),
            PaintColor::Fuchsia => color!(192, 38, 211),
            PaintColor::Empty => color!(115, 115, 115),
            PaintColor::Custom(color) => color,
        }
    }
}

impl From<Color> for PaintColor {
    fn from(value: Color) -> Self {
        PaintColor::Custom(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Shapes {
    Line,
    Bezier,
    Rectangle,
    Circle,
    Triangle,
    Bestagon,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Tool {
    Pencil,
    Eraser,
    Text,
    #[default]
    Brush,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Tool(Tool),
    Select,
    Shape(Shapes),
}

impl Action {
    fn has_opacity(&self) -> bool {
        match self {
            Self::Select => false,
            Self::Shape(_) => true,
            Self::Tool(Tool::Eraser) => false,
            Self::Tool(_) => true,
        }
    }

    fn has_scale(&self) -> bool {
        if let Self::Tool(_) = self {
            true
        } else {
            false
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::Tool(Tool::default())
    }
}

#[derive(Debug, Clone)]
enum Message {
    Action(Action),
    Color(PaintColor),
    Clear,
    Opacity(f32),
    Scale(f32),
    None,
}

#[derive(Debug)]
struct Paint {
    action: Action,
    color: PaintColor,
    palette: [PaintColor; 18],
    opacity: f32,
    scale: f32,
    drawings: Vec<Painting>,
    canvas: State,
}

impl Default for Paint {
    fn default() -> Self {
        let opacity = 1.0;
        let scale = 1.0;
        let color = PaintColor::default();

        let palette = [
            PaintColor::White,
            PaintColor::Black(opacity),
            PaintColor::Grey,
            PaintColor::Ivory,
            PaintColor::Red,
            PaintColor::Orange,
            PaintColor::Yellow,
            PaintColor::Green,
            PaintColor::Blue,
            PaintColor::Indigo,
            PaintColor::Violet,
            PaintColor::Fuchsia,
            PaintColor::Rose,
            PaintColor::Cyan,
            PaintColor::Empty,
            PaintColor::Empty,
            PaintColor::Empty,
            PaintColor::Empty,
        ];

        let mut canvas = State::default();
        canvas.scale(scale);
        canvas.color(color.into());

        Self {
            palette,
            action: Action::default(),
            color,
            opacity,
            scale,
            drawings: Vec::default(),
            canvas,
        }
    }
}

impl Paint {
    fn side_panel(&self) -> Container<'_, Message> {
        let clear = button("Clear")
            .on_press(Message::Clear)
            .style(|theme, status| styles::toolbar_btn(theme, status, false));

        let opacity = {
            let slider =
                vertical_slider(0.0..=1.0, self.opacity, Message::Opacity)
                    .default(1.0)
                    .step(0.05)
                    .shift_step(0.1);

            let desc = text("Opacity").size(15.0);

            tooltip(slider, desc, tooltip::Position::Bottom).gap(8.0)
        };

        let scale = {
            let slider = vertical_slider(0.0..=3.0, self.scale, Message::Scale)
                .default(1.0)
                .step(0.1)
                .shift_step(0.1);

            let desc = text("Scale");

            tooltip(slider, desc, tooltip::Position::Bottom).gap(8.0)
        };

        let mut controls = row!().spacing(10);

        if self.action.has_opacity() {
            controls = controls.push(opacity);
        }

        if self.action.has_scale() {
            controls = controls.push(scale);
        }

        let mut content = column!(clear, controls,)
            .padding([8, 3])
            .align_x(Horizontal::Center);

        if self.action.has_scale() || self.action.has_opacity() {
            content = content.spacing(20.0)
        }

        let content =
            container(content).max_height(400.0).style(styles::controls);

        container(content)
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
    }

    fn colors(&self) -> Column<'_, Message> {
        let description = text("Colors");

        let colors = {
            let mut rw1 = row!().spacing(15);
            let mut rw2 = row!().spacing(15);
            let mut rw3 = row!().spacing(15);

            let colors = self
                .palette
                .iter()
                .map(|color| match color {
                    PaintColor::Empty => (*color, Message::None),
                    _ => (*color, Message::Color(*color)),
                })
                .enumerate();

            for (idx, (color, msg)) in colors {
                let btn = button("").width(20).height(20).on_press(msg).style(
                    move |_, status| styles::color_btn(color.into(), status),
                );

                match idx / 6 {
                    0 => rw1 = rw1.push(btn),
                    1 => rw2 = rw2.push(btn),
                    _ => rw3 = rw3.push(btn),
                }
            }

            column!(rw1, rw2, rw3).spacing(5)
        };

        let current = button("")
            .width(35)
            .height(35)
            .on_press(Message::None)
            .style(|_, status| styles::color_btn(self.color.into(), status));

        let colors =
            row!(current, colors).align_y(Vertical::Center).spacing(10);

        column!(colors, vertical_space(), description)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
    }

    fn toolbar(&self) -> Container<'_, Message> {
        let selector = {
            let icon = text('\u{E847}').size(40.0).font(ICON_FONT);

            let btn = button(icon)
                .on_press(Message::Action(Action::Select))
                .padding([2, 6])
                .style(|theme, status| {
                    styles::toolbar_btn(
                        theme,
                        status,
                        self.action == Action::Select,
                    )
                });

            let description = text("Selection");

            column!(btn, vertical_space(), description)
                .align_x(Horizontal::Center)
                .width(75)
                .height(Length::Fill)
        };

        let tools = {
            let tool_btn = |code: char, message: Message, tool: Tool| {
                let icon = text(code).font(ICON_FONT);

                button(icon).on_press(message).style(move |theme, status| {
                    styles::toolbar_btn(
                        theme,
                        status,
                        self.action == Action::Tool(tool),
                    )
                })
            };

            let rw1 = row!(
                tool_btn(
                    '\u{E800}',
                    Message::Action(Action::Tool(Tool::Pencil)),
                    Tool::Pencil
                ),
                tool_btn(
                    '\u{F12D}',
                    Message::Action(Action::Tool(Tool::Eraser)),
                    Tool::Eraser
                )
            )
            .spacing(2.5);

            let rw2 = row!(
                tool_btn(
                    '\u{E801}',
                    Message::Action(Action::Tool(Tool::Text)),
                    Tool::Text
                ),
                tool_btn(
                    '\u{F1FC}',
                    Message::Action(Action::Tool(Tool::Brush)),
                    Tool::Brush
                )
            )
            .spacing(2.5);

            let description = text("Tools");

            let tools = column!(rw1, rw2).spacing(2.5);

            column!(tools, vertical_space(), description)
                .align_x(Horizontal::Center)
                .height(Length::Fill)
        };

        let shapes = {
            let shape_btn = |code: char, msg: Message, shape: Shapes| {
                let icon = text(code).font(ICON_FONT);

                button(icon).on_press(msg).style(move |theme, status| {
                    styles::toolbar_btn(
                        theme,
                        status,
                        self.action == Action::Shape(shape),
                    )
                })
            };

            let rw1 = row!(
                shape_btn(
                    '\u{E802}',
                    Message::Action(Action::Shape(Shapes::Line)),
                    Shapes::Line
                ),
                shape_btn(
                    '\u{E803}',
                    Message::Action(Action::Shape(Shapes::Bezier)),
                    Shapes::Bezier
                ),
                shape_btn(
                    '\u{E804}',
                    Message::Action(Action::Shape(Shapes::Triangle)),
                    Shapes::Triangle
                ),
            )
            .spacing(2.5);

            let rw2 = row!(
                shape_btn(
                    '\u{E805}',
                    Message::Action(Action::Shape(Shapes::Rectangle)),
                    Shapes::Rectangle
                ),
                shape_btn(
                    '\u{E806}',
                    Message::Action(Action::Shape(Shapes::Circle)),
                    Shapes::Circle
                ),
                shape_btn(
                    '\u{E807}',
                    Message::Action(Action::Shape(Shapes::Bestagon)),
                    Shapes::Bestagon
                ),
            )
            .spacing(2.5);

            let description = text("Shapes");

            let shapes = column!(rw1, rw2).spacing(2.5);

            column!(shapes, vertical_space(), description)
                .align_x(Horizontal::Center)
                .height(Length::Fill)
        };

        container(
            row!(
                selector,
                vertical_rule(2),
                tools,
                vertical_rule(2),
                shapes,
                vertical_rule(2),
                self.colors()
            )
            .width(Length::Fill)
            .height(Length::Fixed(110.0))
            .spacing(10.0)
            .padding([5, 8])
            .align_y(Vertical::Center),
        )
        .style(styles::toolbar)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Action(action) => {
                self.action = action;
                self.canvas.action(action);
            }
            Message::Color(color) => {
                self.color = color;
                self.canvas.color(self.color.opacity(self.opacity).into());
            }
            Message::Clear => {
                self.drawings.clear();
                self.canvas.clear()
            }
            Message::Opacity(opacity) => {
                self.opacity = opacity;
                self.canvas.color(self.color.opacity(self.opacity).into());
            }
            Message::Scale(scale) => {
                self.scale = scale;
                self.canvas.scale(scale);
            }
            Message::None => {}
        }
    }

    fn view(&self) -> Element<Message> {
        let stage = row!(
            self.side_panel(),
            self.canvas
                .view(&self.drawings)
                .map(|_drawing| Message::None)
        )
        .width(Length::Fill)
        .spacing(10.0)
        .padding([6, 6]);

        let content = column!(self.toolbar(), stage);

        container(content).into()
    }
}

mod canvas {

    use iced::{
        advanced::graphics::core::SmolStr,
        color, mouse,
        widget::canvas::{
            self,
            event::{self, Event},
            stroke, Canvas, Frame, Geometry, LineDash, Path, Stroke, Text,
        },
        Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme,
    };

    use super::{Action, Shapes, Tool};

    #[derive(Default, Debug)]
    pub struct State {
        cache: canvas::Cache,
        current_action: Action,
        color: Color,
        scale: f32,
    }

    impl State {
        pub fn clear(&mut self) {
            self.cache.clear()
        }

        pub fn action(&mut self, action: Action) {
            self.current_action = action;
        }

        pub fn color(&mut self, color: Color) {
            self.color = color;
        }

        pub fn scale(&mut self, scale: f32) {
            self.scale = scale;
        }

        pub fn view<'a>(
            &'a self,
            paintings: &'a [Painting],
        ) -> Element<'a, Painting> {
            Canvas::new(PaintingCanvas {
                state: &self,
                paintings,
            })
            .width(Fill)
            .height(Fill)
            .into()
        }
    }

    struct PaintingCanvas<'a> {
        state: &'a State,
        paintings: &'a [Painting],
    }

    impl<'a> canvas::Program<Painting> for PaintingCanvas<'a> {
        type State = Option<Pending>;

        fn update(
            &self,
            state: &mut Self::State,
            event: Event,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        ) -> (event::Status, Option<Painting>) {
            match (cursor.position_in(bounds), state.clone()) {
                (
                    Some(cursor_position),
                    Some(Pending::Text(TextPending::Typing {
                        from,
                        to,
                        text: mut state_text,
                    })),
                ) if self.state.current_action == Action::Tool(Tool::Text) => {
                    match event {
                        Event::Keyboard(
                            iced::keyboard::Event::KeyPressed {
                                text: Some(new_text),
                                ..
                            },
                        ) => {
                            if &new_text == "\u{8}" {
                                state_text.pop();
                            } else {
                                state_text.push_str(&new_text);
                            }

                            state.replace(Pending::Text(TextPending::Typing {
                                from,
                                to,
                                text: state_text,
                            }));

                            return (event::Status::Captured, None);
                        }
                        Event::Mouse(mouse::Event::ButtonPressed(
                            mouse::Button::Left,
                        )) => {
                            let bounds = Rectangle::new(
                                from,
                                Size::new(to.x - from.x, to.y - from.y),
                            );
                            if !bounds.contains(cursor_position) {
                                let painting = Painting::Text {
                                    top_left: from,
                                    bottom_right: to,
                                    text: state_text.clone(),
                                    color: self.state.color,
                                    scale: self.state.scale,
                                };

                                state.take();
                                return (
                                    event::Status::Captured,
                                    Some(painting),
                                );
                            }
                        }

                        _ => {}
                    }
                }

                (
                    _,
                    Some(Pending::Text(TextPending::Typing {
                        text: mut state_text,
                        from,
                        to,
                    })),
                ) => match event {
                    Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        text: Some(new_text),
                        ..
                    }) => {
                        state_text.push_str(&new_text);

                        state.replace(Pending::Text(TextPending::Typing {
                            from,
                            to,
                            text: state_text,
                        }));

                        return (event::Status::Captured, None);
                    }
                    _ => {}
                },

                (Some(cursor_position), _unused_state) => match event {
                    Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    )) if self.state.current_action
                        == Action::Tool(Tool::Text) =>
                    {
                        match state {
                            Some(Pending::Text(TextPending::One { from })) => {
                                let typing =
                                    Pending::Text(TextPending::Typing {
                                        from: *from,
                                        to: cursor_position,
                                        text: String::default(),
                                    });

                                state.replace(typing);
                                return (event::Status::Captured, None);
                            }
                            Some(_) => {
                                panic!("Drawing while typing tool is selected")
                            }
                            None => {}
                        }
                    }

                    Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    )) if self.state.current_action
                        == Action::Shape(Shapes::Bezier) =>
                    {
                        match state {
                            Some(Pending::Drawing(DrawingPending::One {
                                from,
                            })) => {
                                let pending =
                                    Pending::Drawing(DrawingPending::Two {
                                        from: *from,
                                        to: cursor_position,
                                    });

                                state.replace(pending);
                                return (event::Status::Captured, None);
                            }
                            Some(Pending::Text(_)) => {
                                panic!("Typing while bezier tool is selected")
                            }
                            _ => {}
                        }
                    }

                    Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    )) => match state {
                        Some(Pending::Drawing(DrawingPending::One {
                            from,
                        })) => {
                            let painting = Painting::new(
                                self.state.current_action,
                                *from,
                                cursor_position,
                                self.state.color,
                                self.state.scale,
                            );
                            state.take();
                            return (event::Status::Captured, Some(painting));
                        }
                        Some(Pending::Drawing(DrawingPending::Two {
                            from,
                            ..
                        })) => {
                            let painting = Painting::new(
                                self.state.current_action,
                                *from,
                                cursor_position,
                                self.state.color,
                                self.state.scale,
                            );
                            state.take();
                            return (event::Status::Captured, Some(painting));
                        }
                        Some(Pending::Text(_)) => {
                            panic!("Typing when text tool not selected")
                        }

                        Some(Pending::Drawing(DrawingPending::Bezier {
                            ..
                        })) => {}

                        None => {}
                    },

                    Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    )) => match state {
                        Some(Pending::Drawing(DrawingPending::Two {
                            from,
                            to,
                        })) if self.state.current_action
                            == Action::Shape(Shapes::Bezier) =>
                        {
                            let painting = Painting::Bezier {
                                from: *from,
                                to: *to,
                                control: cursor_position,
                                scale: self.state.scale,
                                color: self.state.color,
                            };
                            state.take();
                            return (event::Status::Captured, Some(painting));
                        }
                        Some(Pending::Drawing(DrawingPending::Bezier {
                            from,
                            to,
                            ..
                        })) => {
                            let painting = Painting::Bezier {
                                from: *from,
                                to: *to,
                                control: cursor_position,
                                scale: self.state.scale,
                                color: self.state.color,
                            };
                            state.take();
                            return (event::Status::Captured, Some(painting));
                        }
                        Some(Pending::Text(TextPending::Typing {
                            from,
                            to,
                            text,
                        })) if self.state.current_action
                            == Action::Tool(Tool::Text) =>
                        {
                            let bounds = Rectangle::new(
                                *from,
                                Size::new(to.x - from.x, to.y - from.y),
                            );
                            if !bounds.contains(cursor_position) {
                                let painting = Painting::Text {
                                    top_left: *from,
                                    bottom_right: *to,
                                    text: text.clone(),
                                    color: self.state.color,
                                    scale: self.state.scale,
                                };

                                state.take();
                                return (
                                    event::Status::Captured,
                                    Some(painting),
                                );
                            }
                        }
                        Some(_) => {}
                        None => {
                            let pending = if self.state.current_action
                                == Action::Tool(Tool::Text)
                            {
                                Pending::Text(TextPending::One {
                                    from: cursor_position,
                                })
                            } else {
                                Pending::Drawing(DrawingPending::One {
                                    from: cursor_position,
                                })
                            };

                            state.replace(pending);

                            return (event::Status::Captured, None);
                        }
                    },

                    _ => {}
                },
                _ => {}
            };

            return (event::Status::Ignored, None);
        }

        fn draw(
            &self,
            state: &Self::State,
            renderer: &Renderer,
            theme: &Theme,
            bounds: Rectangle,
            cursor: iced::advanced::mouse::Cursor,
        ) -> Vec<Geometry<Renderer>> {
            let content =
                self.state.cache.draw(renderer, bounds.size(), |frame| {
                    Painting::draw_all(self.paintings, frame, theme);

                    frame.fill_rectangle(
                        Point::ORIGIN,
                        frame.size(),
                        color!(240, 234, 214),
                    );
                });

            if let Some(pending) = state {
                vec![
                    content,
                    pending.draw(
                        renderer,
                        theme,
                        bounds,
                        cursor,
                        self.state.current_action,
                        self.state.color,
                        self.state.scale,
                    ),
                ]
            } else {
                vec![content]
            }
        }

        fn mouse_interaction(
            &self,
            state: &Self::State,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        ) -> mouse::Interaction {
            match state {
                Some(Pending::Text(TextPending::One { .. }))
                    if cursor.is_over(bounds) =>
                {
                    mouse::Interaction::Text
                }
                Some(_) | None if cursor.is_over(bounds) => {
                    mouse::Interaction::Crosshair
                }

                _ => mouse::Interaction::default(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Painting {
        FreeForm {
            color: Color,
            scale: f32,
        },
        Text {
            top_left: Point,
            bottom_right: Point,
            text: String,
            color: Color,
            scale: f32,
        },
        Line {
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        },
        Bezier {
            from: Point,
            to: Point,
            control: Point,
            color: Color,
            scale: f32,
        },
        Rectangle {
            top_left: Point,
            bottom_right: Point,
            color: Color,
            scale: f32,
        },
        Circle {
            center: Point,
            radius: Point,
            color: Color,
            scale: f32,
        },
        Triangle {
            left: Point,
            right: Point,
            color: Color,
            scale: f32,
        },
        Bestagon {
            top_left: Point,
            bottom_right: Point,
            color: Color,
            scale: f32,
        },
        Eraser {
            scale: f32,
        },
        Select {
            top_left: Point,
            bottom_right: Point,
            color: Color,
            scale: f32,
        },
    }

    impl Painting {
        fn new(
            action: Action,
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        ) -> Self {
            match action {
                Action::Tool(Tool::Text) => Self::Text {
                    top_left: from,
                    bottom_right: to,
                    text: String::from("Text painting here invalid"),
                    color,
                    scale,
                },
                Action::Tool(Tool::Brush) => Self::FreeForm { color, scale },
                Action::Tool(Tool::Pencil) => Self::FreeForm { color, scale },
                Action::Tool(Tool::Eraser) => Self::Eraser { scale },
                Action::Select => Self::Select {
                    top_left: from,
                    bottom_right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Rectangle) => Self::Rectangle {
                    top_left: from,
                    bottom_right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Line) => Self::Line {
                    from,
                    to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Triangle) => Self::Triangle {
                    left: from,
                    right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Circle) => Self::Circle {
                    center: from,
                    radius: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Bestagon) => Self::Bestagon {
                    top_left: from,
                    bottom_right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Bezier) => Self::Bezier {
                    from,
                    to,
                    control: to,
                    color,
                    scale,
                },
            }
        }

        fn draw_all(_paintings: &[Self], _frame: &mut Frame, _theme: &Theme) {}
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Pending {
        Text(TextPending),
        Drawing(DrawingPending),
    }

    impl Pending {
        fn draw(
            &self,
            renderer: &Renderer,
            theme: &Theme,
            bounds: Rectangle,
            cursor: mouse::Cursor,
            action: Action,
            color: Color,
            scale: f32,
        ) -> Geometry {
            match self {
                Self::Text(text) => {
                    text.draw(renderer, bounds, cursor, color, scale)
                }
                Self::Drawing(drawing) => drawing.draw(
                    renderer, theme, bounds, cursor, action, color, scale,
                ),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum DrawingPending {
        One {
            from: Point,
        },
        Two {
            from: Point,
            to: Point,
        },
        Bezier {
            from: Point,
            to: Point,
            control: Point,
        },
    }

    impl DrawingPending {
        fn draw(
            &self,
            renderer: &Renderer,
            _theme: &Theme,
            bounds: Rectangle,
            _cursor: mouse::Cursor,
            _action: Action,
            _color: Color,
            _scale: f32,
        ) -> Geometry {
            let frame = Frame::new(renderer, bounds.size());

            frame.into_geometry()
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TextPending {
        One {
            from: Point,
        },
        Typing {
            from: Point,
            to: Point,
            text: String,
        },
    }

    impl TextPending {
        fn draw(
            &self,
            renderer: &Renderer,
            bounds: Rectangle,
            cursor: mouse::Cursor,
            color: Color,
            scale: f32,
        ) -> Geometry {
            let mut frame = Frame::new(renderer, bounds.size());

            let line_dash = LineDash {
                offset: 0,
                segments: &[4.0, 0.0, 4.0],
            };

            let stroke = Stroke {
                line_dash,
                style: stroke::Style::Solid(color),
                width: 2.0,
                ..Default::default()
            };

            match self {
                Self::One { from } => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        let size = Size::new(
                            cursor_position.x - from.x,
                            cursor_position.y - from.y,
                        );
                        let rect = Path::rectangle(*from, size);
                        frame.stroke(&rect, stroke);
                    }
                }
                Self::Typing { from, to, text } => {
                    let size = Size::new(to.x - from.x, to.y - from.y);
                    let rect = Path::rectangle(*from, size);
                    frame.stroke(&rect, stroke);

                    let size = (16.0 * scale.max(0.1)).into();

                    let position = {
                        let left = bounds.width * 0.005;
                        let top = bounds.height * 0.005;

                        Point::new(from.x + left, from.y + top)
                    };

                    let text = Text {
                        content: text.clone(),
                        position,
                        color,
                        size,
                        ..Default::default()
                    };

                    frame.fill_text(text)
                }
            }

            frame.into_geometry()
        }
    }
}

mod styles {
    use iced::{widget, Background, Border, Color, Theme};

    pub fn toolbar(theme: &Theme) -> widget::container::Style {
        let background = theme.extended_palette().background.weak;

        widget::container::Style {
            background: Some(Background::Color(background.color)),
            text_color: Some(background.text),
            ..Default::default()
        }
    }

    pub fn controls(theme: &Theme) -> widget::container::Style {
        widget::container::Style {
            border: Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            ..toolbar(theme)
        }
    }

    pub fn toolbar_btn(
        theme: &Theme,
        status: widget::button::Status,
        selected: bool,
    ) -> widget::button::Style {
        let background = match status {
            widget::button::Status::Hovered => {
                theme.extended_palette().background.strong
            }
            _status if selected => theme.extended_palette().background.strong,
            _ => theme.extended_palette().background.weak,
        };

        widget::button::Style {
            background: Some(Background::Color(background.color)),
            border: Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            text_color: background.text,
            ..Default::default()
        }
    }

    pub fn color_btn(
        color: Color,
        status: widget::button::Status,
    ) -> widget::button::Style {
        let background = color;

        match status {
            widget::button::Status::Hovered => widget::button::Style {
                background: Some(Background::Color(background)),
                border: Border {
                    width: 0.0,
                    radius: 100.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => widget::button::Style {
                background: Some(Background::Color(background)),
                border: Border {
                    width: 0.5,
                    radius: 100.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
