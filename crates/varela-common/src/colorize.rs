//! [owo-colors](https://github.com/jam1garner/owo-colors) stripped down to only ANSI color and converted over to [`vfmt::uDebug`] and [`vfmt::uDisplay`].

use std::marker::PhantomData;

/// A trait for describing a type which can be used with [`FgColorDisplay`](FgColorDisplay) or
/// [`BgCBgColorDisplay`](BgColorDisplay)
pub trait Color {
    /// The ANSI format code for setting this color as the foreground
    const ANSI_FG: &'static str;

    /// The ANSI format code for setting this color as the background
    const ANSI_BG: &'static str;

    /// The raw ANSI format for settings this color as the foreground without the ANSI
    /// delimiters ("\x1b" and "m")
    const RAW_ANSI_FG: &'static str;

    /// The raw ANSI format for settings this color as the background without the ANSI
    /// delimiters ("\x1b" and "m")
    const RAW_ANSI_BG: &'static str;

    #[doc(hidden)]
    type DynEquivelant: DynColor;

    #[doc(hidden)]
    const DYN_EQUIVELANT: Self::DynEquivelant;

    #[doc(hidden)]
    fn into_dyncolors() -> crate::colorize::dyn_colors::DynColors;
}

/// A trait describing a runtime-configurable color which can displayed using [`FgDynColorDisplay`](FgDynColorDisplay)
/// or [`BgDynColorDisplay`](BgDynColorDisplay). If your color will be known at compile time it
/// is recommended you avoid this.
pub trait DynColor {
    /// A function to output a ANSI code to a formatter to set the foreground to this color
    fn fmt_ansi_fg<W: vfmt::uWrite + ?Sized>(
        &self,
        f: &mut vfmt::Formatter<'_, W>,
    ) -> Result<(), W::Error>;
    /// A function to output a ANSI code to a formatter to set the background to this color
    fn fmt_ansi_bg<W: vfmt::uWrite + ?Sized>(
        &self,
        f: &mut vfmt::Formatter<'_, W>,
    ) -> Result<(), W::Error>;

    /// A function to output a raw ANSI code to a formatter to set the foreground to this color,
    /// but without including the ANSI delimiters.
    fn fmt_raw_ansi_fg<W: vfmt::uWrite + ?Sized>(
        &self,
        f: &mut vfmt::Formatter<'_, W>,
    ) -> Result<(), W::Error>;

    /// A function to output a raw ANSI code to a formatter to set the background to this color,
    /// but without including the ANSI delimiters.
    fn fmt_raw_ansi_bg<W: vfmt::uWrite + ?Sized>(
        &self,
        f: &mut vfmt::Formatter<'_, W>,
    ) -> Result<(), W::Error>;

    #[doc(hidden)]
    fn get_dyncolors_fg(&self) -> crate::colorize::dyn_colors::DynColors;
    #[doc(hidden)]
    fn get_dyncolors_bg(&self) -> crate::colorize::dyn_colors::DynColors;
}

/// Transparent wrapper around a type which implements all the formatters the wrapped type does,
/// with the addition of changing the foreground color. Recommended to be constructed using
/// [`Colorize`](Colorize).
#[repr(transparent)]
pub struct FgColorDisplay<'a, C: Color, T>(&'a T, PhantomData<C>);

impl<'a, C: Color, T> std::ops::Deref for FgColorDisplay<'a, C, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Transparent wrapper around a type which implements all the formatters the wrapped type does,
/// with the addition of changing the background color. Recommended to be constructed using
/// [`Colorize`](Colorize).
#[repr(transparent)]
pub struct BgColorDisplay<'a, C: Color, T>(&'a T, PhantomData<C>);

/// Wrapper around a type which implements all the formatters the wrapped type does,
/// with the addition of changing the foreground color. Is not recommended unless compile-time
/// coloring is not an option.
pub struct FgDynColorDisplay<'a, Color: DynColor, T>(&'a T, Color);

/// Wrapper around a type which implements all the formatters the wrapped type does,
/// with the addition of changing the background color. Is not recommended unless compile-time
/// coloring is not an option.
pub struct BgDynColorDisplay<'a, Color: DynColor, T>(&'a T, Color);

macro_rules! style_methods {
    ($(#[$meta:meta] $name:ident $ty:ident),* $(,)?) => {
        $(
            #[$meta]
            #[must_use]
            #[inline(always)]
            fn $name(&self) -> styles::$ty<'_, Self> {
                styles::$ty(self)
            }
         )*
    };
}

macro_rules! color_methods {
    ($(
        #[$fg_meta:meta] #[$bg_meta:meta] $color:ident $fg_method:ident $bg_method:ident
    ),* $(,)?) => {
        $(
            #[$fg_meta]
            #[must_use]
            #[inline(always)]
            fn $fg_method(&self) -> FgColorDisplay<'_, colors::$color, Self> {
                FgColorDisplay(self, PhantomData)
            }

            #[$bg_meta]
            #[must_use]
            #[inline(always)]
            fn $bg_method(&self) -> BgColorDisplay<'_, colors::$color, Self> {
                BgColorDisplay(self, PhantomData)
            }
         )*
    };
}

/// Extension trait for colorizing a type which implements any std formatter
/// ([`Display`](core::vfmt::uDisplay), [`Debug`](core::vfmt::uDebug), [`UpperHex`](core::fmt::UpperHex),
/// etc.)
///
/// ## Example
///
/// ```rust
/// use owo_colors::Colorize;
///
/// println!("My number is {:#x}!", 10.green());
/// println!("My number is not {}!", 4.on_red());
/// ```
///
/// ## How to decide which method to use
///
/// **Do you have a specific color you want to use?**
///
/// Use the specific color's method, such as [`blue`](Colorize::blue) or
/// [`on_green`](Colorize::on_green).
///
///
/// **Do you want your colors configurable via generics?**
///
/// Use [`fg`](Colorize::fg) and [`bg`](Colorize::bg) to make it compile-time configurable.
///
///
/// **Do you need to pick a color at runtime?**
///
/// Use the [`color`](Colorize::color), [`on_color`](Colorize::on_color),
/// [`truecolor`](Colorize::truecolor) or [`on_truecolor`](Colorize::on_truecolor).
///
/// **Do you need some other text modifier?**
///
/// * [`bold`](Colorize::bold)
/// * [`dimmed`](Colorize::dimmed)
/// * [`italic`](Colorize::italic)
/// * [`underline`](Colorize::underline)
/// * [`blink`](Colorize::blink)
/// * [`blink_fast`](Colorize::blink_fast)
/// * [`reversed`](Colorize::reversed)
/// * [`hidden`](Colorize::hidden)
/// * [`strikethrough`](Colorize::strikethrough)
///
/// **Do you want it to only display colors if it's a terminal?**
///
/// 1. Enable the `supports-colors` feature
/// 2. Colorize inside [`if_supports_color`](Colorize::if_supports_color)
///
/// **Do you need to store a set of colors/effects to apply to multiple things?**
///
/// Use [`style`](Colorize::style) to apply a [`Style`]
///
pub trait Colorize: Sized {
    /// Set the foreground color generically
    ///
    /// ```rust
    /// use owo_colors::{Colorize, colors::*};
    ///
    /// println!("{}", "red foreground".fg::<Red>());
    /// ```
    #[must_use]
    #[inline(always)]
    fn fg<C: Color>(&self) -> FgColorDisplay<'_, C, Self> {
        FgColorDisplay(self, PhantomData)
    }

    /// Set the background color generically.
    ///
    /// ```rust
    /// use owo_colors::{Colorize, colors::*};
    ///
    /// println!("{}", "black background".bg::<Black>());
    /// ```
    #[must_use]
    #[inline(always)]
    fn bg<C: Color>(&self) -> BgColorDisplay<'_, C, Self> {
        BgColorDisplay(self, PhantomData)
    }

    color_methods! {
        /// Change the foreground color to black
        /// Change the background color to black
        Black    black    on_black,
        /// Change the foreground color to red
        /// Change the background color to red
        Red      red      on_red,
        /// Change the foreground color to green
        /// Change the background color to green
        Green    green    on_green,
        /// Change the foreground color to yellow
        /// Change the background color to yellow
        Yellow   yellow   on_yellow,
        /// Change the foreground color to blue
        /// Change the background color to blue
        Blue     blue     on_blue,
        /// Change the foreground color to magenta
        /// Change the background color to magenta
        Magenta  magenta  on_magenta,
        /// Change the foreground color to purple
        /// Change the background color to purple
        Magenta  purple   on_purple,
        /// Change the foreground color to cyan
        /// Change the background color to cyan
        Cyan     cyan     on_cyan,
        /// Change the foreground color to white
        /// Change the background color to white
        White    white    on_white,

        /// Change the foreground color to the terminal default
        /// Change the background color to the terminal default
        Default default_color on_default_color,

        /// Change the foreground color to bright black
        /// Change the background color to bright black
        BrightBlack    bright_black    on_bright_black,
        /// Change the foreground color to bright red
        /// Change the background color to bright red
        BrightRed      bright_red      on_bright_red,
        /// Change the foreground color to bright green
        /// Change the background color to bright green
        BrightGreen    bright_green    on_bright_green,
        /// Change the foreground color to bright yellow
        /// Change the background color to bright yellow
        BrightYellow   bright_yellow   on_bright_yellow,
        /// Change the foreground color to bright blue
        /// Change the background color to bright blue
        BrightBlue     bright_blue     on_bright_blue,
        /// Change the foreground color to bright magenta
        /// Change the background color to bright magenta
        BrightMagenta  bright_magenta  on_bright_magenta,
        /// Change the foreground color to bright purple
        /// Change the background color to bright purple
        BrightMagenta  bright_purple   on_bright_purple,
        /// Change the foreground color to bright cyan
        /// Change the background color to bright cyan
        BrightCyan     bright_cyan     on_bright_cyan,
        /// Change the foreground color to bright white
        /// Change the background color to bright white
        BrightWhite    bright_white    on_bright_white,
    }

    style_methods! {
        /// Make the text bold
        bold BoldDisplay,
        /// Make the text dim
        dimmed DimDisplay,
        /// Make the text italicized
        italic ItalicDisplay,
        /// Make the text italicized
        underline UnderlineDisplay,
        /// Make the text blink
        blink BlinkDisplay,
        /// Make the text blink (but fast!)
        blink_fast BlinkFastDisplay,
        /// Swap the foreground and background colors
        reversed ReversedDisplay,
        /// Hide the text
        hidden HiddenDisplay,
        /// Cross out the text
        strikethrough StrikeThroughDisplay,
    }

    /// Set the foreground color at runtime. Only use if you do not know which color will be used at
    /// compile-time. If the color is constant, use either [`Colorize::fg`](Colorize::fg) or
    /// a color-specific method, such as [`Colorize::green`](Colorize::green),
    ///
    /// ```rust
    /// use owo_colors::{Colorize, AnsiColors};
    ///
    /// println!("{}", "green".color(AnsiColors::Green));
    /// ```
    #[must_use]
    #[inline(always)]
    fn color<Color: DynColor>(&self, color: Color) -> FgDynColorDisplay<'_, Color, Self> {
        FgDynColorDisplay(self, color)
    }

    /// Set the background color at runtime. Only use if you do not know what color to use at
    /// compile-time. If the color is constant, use either [`Colorize::bg`](Colorize::bg) or
    /// a color-specific method, such as [`Colorize::on_yellow`](Colorize::on_yellow),
    ///
    /// ```rust
    /// use owo_colors::{Colorize, AnsiColors};
    ///
    /// println!("{}", "yellow background".on_color(AnsiColors::BrightYellow));
    /// ```
    #[must_use]
    #[inline(always)]
    fn on_color<Color: DynColor>(&self, color: Color) -> BgDynColorDisplay<'_, Color, Self> {
        BgDynColorDisplay(self, color)
    }
}

impl<D: Sized> Colorize for D {}

pub mod colors {
    //! Color types for used for being generic over the color

    use crate::colorize::{BgColorDisplay, BgDynColorDisplay, FgColorDisplay, FgDynColorDisplay};

    macro_rules! colors {
        ($(
            $color:ident $fg:literal $bg:literal
        ),* $(,)?) => {

            pub(crate) mod ansi_colors {
                #[allow(unused_imports)]
                use crate::colorize::Colorize;

                /// Available standard ANSI colors for use with [`Colorize::color`](Colorize::color)
                /// or [`Colorize::on_color`](Colorize::on_color)
                #[allow(missing_docs)]
                #[derive(Copy, Clone, Debug, PartialEq)]
                pub enum AnsiColors {
                    $(
                        $color,
                    )*
                }

                impl crate::colorize::DynColor for AnsiColors {
                    fn fmt_ansi_fg<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                        let color = match self {
                            $(
                                AnsiColors::$color => concat!("\x1b[", stringify!($fg), "m"),
                            )*
                        };

                        // vfmt::uwrite!(f, "{}", color)
                        f.write_str(color)
                    }

                    fn fmt_ansi_bg<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                        let color = match self {
                            $(
                                AnsiColors::$color => concat!("\x1b[", stringify!($bg), "m"),
                            )*
                        };

                        // vfmt::uwrite!(f, "{}", color)
                        f.write_str(color)
                    }

                    fn fmt_raw_ansi_fg<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                        let color = match self {
                            $(
                                AnsiColors::$color => stringify!($fg),
                            )*
                        };

                        f.write_str(color)
                    }

                    fn fmt_raw_ansi_bg<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                        let color = match self {
                            $(
                                AnsiColors::$color => stringify!($bg),
                            )*
                        };

                        f.write_str(color)
                    }

                    #[doc(hidden)]
                    fn get_dyncolors_fg(&self) -> crate::colorize::dyn_colors::DynColors {
                        crate::colorize::dyn_colors::DynColors::Ansi(*self)
                    }

                    #[doc(hidden)]
                    fn get_dyncolors_bg(&self) -> crate::colorize::dyn_colors::DynColors {
                        crate::colorize::dyn_colors::DynColors::Ansi(*self)
                    }
                }
            }

            $(
                /// A color for use with [`Colorize`](crate::colorize::Colorize)'s `fg` and `bg` methods.
                pub struct $color;

                impl crate::colorize::Color for $color {
                    const ANSI_FG: &'static str = concat!("\x1b[", stringify!($fg), "m");
                    const ANSI_BG: &'static str = concat!("\x1b[", stringify!($bg), "m");

                    const RAW_ANSI_FG: &'static str = stringify!($fg);
                    const RAW_ANSI_BG: &'static str = stringify!($bg);

                    #[doc(hidden)]
                    type DynEquivelant = ansi_colors::AnsiColors;

                    #[doc(hidden)]
                    const DYN_EQUIVELANT: Self::DynEquivelant = ansi_colors::AnsiColors::$color;

                    #[doc(hidden)]
                    fn into_dyncolors() -> crate::colorize::dyn_colors::DynColors {
                        crate::colorize::dyn_colors::DynColors::Ansi(ansi_colors::AnsiColors::$color)
                    }
                }
            )*

        };
    }

    colors! {
        Black   30 40,
        Red     31 41,
        Green   32 42,
        Yellow  33 43,
        Blue    34 44,
        Magenta 35 45,
        Cyan    36 46,
        White   37 47,
        Default   39 49,

        BrightBlack   90 100,
        BrightRed     91 101,
        BrightGreen   92 102,
        BrightYellow  93 103,
        BrightBlue    94 104,
        BrightMagenta 95 105,
        BrightCyan    96 106,
        BrightWhite   97 107,
    }

    macro_rules! impl_fmt_for {
        ($($trait:path),* $(,)?) => {
            $(
                impl<'a, Color: crate::colorize::Color, T: $trait> $trait for FgColorDisplay<'a, Color, T> {
                    #[inline(always)]
                    fn fmt<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                        f.write_str(Color::ANSI_FG)?;
                        <T as $trait>::fmt(&self.0, f)?;
                        f.write_str("\x1b[39m")
                    }
                }

                impl<'a, Color: crate::colorize::Color, T: $trait> $trait for BgColorDisplay<'a, Color, T> {
                    #[inline(always)]
                    fn fmt<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                        f.write_str(Color::ANSI_BG)?;
                        <T as $trait>::fmt(&self.0, f)?;
                        f.write_str("\x1b[49m")
                    }
                }
            )*
        };
    }

    impl_fmt_for! {
        vfmt::uDisplay,
        vfmt::uDebug,
    }

    macro_rules! impl_fmt_for_dyn {
        ($($trait:path),* $(,)?) => {
            $(
                impl<'a, Color: crate::colorize::DynColor, T: $trait> $trait for FgDynColorDisplay<'a, Color, T> {
                    #[inline(always)]
                    fn fmt<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                        (self.1).fmt_ansi_fg(f)?;
                        <T as $trait>::fmt(&self.0, f)?;
                        f.write_str("\x1b[39m")
                    }
                }

                impl<'a, Color: crate::colorize::DynColor, T: $trait> $trait for BgDynColorDisplay<'a, Color, T> {
                    #[inline(always)]
                    fn fmt<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                        (self.1).fmt_ansi_bg(f)?;
                        <T as $trait>::fmt(&self.0, f)?;
                        f.write_str("\x1b[49m")
                    }
                }
            )*
        };
    }

    impl_fmt_for_dyn! {
        vfmt::uDisplay,
        vfmt::uDebug,
    }
}

mod dyn_colors {
    use crate::colorize::{colors::ansi_colors::AnsiColors, DynColor};

    #[allow(missing_docs)]
    #[derive(Copy, Clone, PartialEq, Debug)]
    pub enum DynColors {
        Ansi(AnsiColors),
    }

    impl DynColor for DynColors {
        fn fmt_ansi_fg<W: vfmt::uWrite + ?Sized>(
            &self,
            f: &mut vfmt::Formatter<'_, W>,
        ) -> Result<(), W::Error> {
            match self {
                DynColors::Ansi(ansi) => ansi.fmt_ansi_fg(f),
            }
        }

        fn fmt_ansi_bg<W: vfmt::uWrite + ?Sized>(
            &self,
            f: &mut vfmt::Formatter<'_, W>,
        ) -> Result<(), W::Error> {
            match self {
                DynColors::Ansi(ansi) => ansi.fmt_ansi_bg(f),
            }
        }

        fn fmt_raw_ansi_fg<W: vfmt::uWrite + ?Sized>(
            &self,
            f: &mut vfmt::Formatter<'_, W>,
        ) -> Result<(), W::Error> {
            match self {
                DynColors::Ansi(ansi) => ansi.fmt_raw_ansi_fg(f),
            }
        }

        fn fmt_raw_ansi_bg<W: vfmt::uWrite + ?Sized>(
            &self,
            f: &mut vfmt::Formatter<'_, W>,
        ) -> Result<(), W::Error> {
            match self {
                DynColors::Ansi(ansi) => ansi.fmt_raw_ansi_bg(f),
            }
        }

        #[doc(hidden)]
        fn get_dyncolors_fg(&self) -> crate::colorize::dyn_colors::DynColors {
            *self
        }

        #[doc(hidden)]
        fn get_dyncolors_bg(&self) -> crate::colorize::dyn_colors::DynColors {
            *self
        }
    }

    /// An error for when the color can not be parsed from a string at runtime
    #[derive(Debug)]
    pub struct ParseColorError;

    impl core::str::FromStr for DynColors {
        type Err = ParseColorError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let ansi = match s {
                "black" => AnsiColors::Black,
                "red" => AnsiColors::Red,
                "green" => AnsiColors::Green,
                "yellow" => AnsiColors::Yellow,
                "blue" => AnsiColors::Blue,
                "magenta" => AnsiColors::Magenta,
                "purple" => AnsiColors::Magenta,
                "cyan" => AnsiColors::Cyan,
                "white" => AnsiColors::White,
                "bright black" => AnsiColors::BrightBlack,
                "bright red" => AnsiColors::BrightRed,
                "bright green" => AnsiColors::BrightGreen,
                "bright yellow" => AnsiColors::BrightYellow,
                "bright blue" => AnsiColors::BrightBlue,
                "bright magenta" => AnsiColors::BrightMagenta,
                "bright cyan" => AnsiColors::BrightCyan,
                "bright white" => AnsiColors::BrightWhite,
                _ => return Err(ParseColorError),
            };

            Ok(Self::Ansi(ansi))
        }
    }
}

pub mod styles {
    //! Different display styles (strikethrough, bold, etc.)

    #[allow(unused_imports)]
    use crate::colorize::Colorize;

    macro_rules! impl_fmt_for_style {
    ($(($ty:ident, $trait:path, $ansi:literal)),* $(,)?) => {
        $(
            impl<'a, T: $trait> $trait for $ty<'a, T> {
                fn fmt<W: vfmt::uWrite + ?Sized>(&self, f: &mut vfmt::Formatter<'_, W>) -> Result<(), W::Error> {
                    f.write_str($ansi)?;
                    <_ as $trait>::fmt(&self.0, f)?;
                    f.write_str("\x1b[0m")
                }
            }
        )*
    };
}

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of boldening it. Recommended to be constructed using
    /// [`Colorize`](Colorize::bold).
    #[repr(transparent)]
    pub struct BoldDisplay<'a, T>(pub &'a T);

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of dimming it. Recommended to be constructed using
    /// [`Colorize`](Colorize::dimmed).
    #[repr(transparent)]
    pub struct DimDisplay<'a, T>(pub &'a T);

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of italicizing it. Recommended to be constructed using
    /// [`Colorize`](Colorize::italic).
    #[repr(transparent)]
    pub struct ItalicDisplay<'a, T>(pub &'a T);

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of underlining it. Recommended to be constructed using
    /// [`Colorize`](Colorize::underline).
    #[repr(transparent)]
    pub struct UnderlineDisplay<'a, T>(pub &'a T);

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of making it blink. Recommended to be constructed using
    /// [`Colorize`](Colorize::blink).
    #[repr(transparent)]
    pub struct BlinkDisplay<'a, T>(pub &'a T);

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of making it blink fast. Recommended to be constructed using
    /// [`Colorize`](Colorize::blink_fast).
    #[repr(transparent)]
    pub struct BlinkFastDisplay<'a, T>(pub &'a T);

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of swapping foreground and background colors. Recommended to be constructed
    /// using [`Colorize`](Colorize::reversed).
    #[repr(transparent)]
    pub struct ReversedDisplay<'a, T>(pub &'a T);

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of hiding the text. Recommended to be constructed
    /// using [`Colorize`](Colorize::reversed).
    #[repr(transparent)]
    pub struct HiddenDisplay<'a, T>(pub &'a T);

    /// Transparent wrapper around a type which implements all the formatters the wrapped type does,
    /// with the addition of crossing out the given text. Recommended to be constructed using
    /// [`Colorize`](Colorize::strikethrough).
    #[repr(transparent)]
    pub struct StrikeThroughDisplay<'a, T>(pub &'a T);

    impl_fmt_for_style! {
        // Bold
        (BoldDisplay, vfmt::uDisplay,  "\x1b[1m"),
        (BoldDisplay, vfmt::uDebug,    "\x1b[1m"),

        // Dim
        (DimDisplay, vfmt::uDisplay,  "\x1b[2m"),
        (DimDisplay, vfmt::uDebug,    "\x1b[2m"),

        // Italic
        (ItalicDisplay, vfmt::uDisplay,  "\x1b[3m"),
        (ItalicDisplay, vfmt::uDebug,    "\x1b[3m"),

        // Underline
        (UnderlineDisplay, vfmt::uDisplay,  "\x1b[4m"),
        (UnderlineDisplay, vfmt::uDebug,    "\x1b[4m"),

        // Blink
        (BlinkDisplay, vfmt::uDisplay,  "\x1b[5m"),
        (BlinkDisplay, vfmt::uDebug,    "\x1b[5m"),

        // Blink fast
        (BlinkFastDisplay, vfmt::uDisplay,  "\x1b[6m"),
        (BlinkFastDisplay, vfmt::uDebug,    "\x1b[6m"),

        // Reverse video
        (ReversedDisplay, vfmt::uDisplay,  "\x1b[7m"),
        (ReversedDisplay, vfmt::uDebug,    "\x1b[7m"),

        // Hide the text
        (HiddenDisplay, vfmt::uDisplay,  "\x1b[8m"),
        (HiddenDisplay, vfmt::uDebug,    "\x1b[8m"),

        // StrikeThrough
        (StrikeThroughDisplay, vfmt::uDisplay,  "\x1b[9m"),
        (StrikeThroughDisplay, vfmt::uDebug,    "\x1b[9m"),
    }
}
