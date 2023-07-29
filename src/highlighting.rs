use termion::color;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Escape,
    Character,
    Comment,
    PrimaryKey,
    SecondaryKey,
}

impl Type {
    pub fn to_color(&self) -> impl color::Color {
        match self {
            Type::Number => color::Rgb(220, 163, 163),
            Type::Match => color::Rgb(38, 139, 210),
            Type::String => color::Rgb(211, 54, 130),
            Type::Escape => color::Rgb(255, 255, 0),
            Type::Character => color::Rgb(108, 113, 196),
            Type::Comment => color::Rgb(128,128,128),
            Type::PrimaryKey => color::Rgb(0, 245, 255),
            Type::SecondaryKey => color::Rgb(255, 99, 71),
            Type::None => color::Rgb(255, 255, 255),
        }
    }
}