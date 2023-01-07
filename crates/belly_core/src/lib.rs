use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::text::TextLayoutInfo;
use bevy::utils::HashMap;
use bevy::{ecs::system::EntityCommands, prelude::*};
use eml::build::BuildPligin;
use eml::EmlPlugin;
use ess::{EssPlugin, StyleSheet, StyleSheetParser};
use input::ElementsInputPlugin;
use std::error::Error;
use std::fmt::Display;
use std::sync::{Arc, RwLock};
// use focus::{Focused, update_focus};
use property::{CompoundProperty, PropertyValue};

pub mod element;
pub mod eml;
pub mod ess;
pub mod input;
pub mod params;
pub mod property;
pub mod relations;
pub mod tags;
pub mod variant;

pub struct ElementsCorePlugin;

pub use crate::eml::build::ElementBuilder;
pub use crate::eml::build::ElementBuilderRegistry;
pub use crate::eml::build::ElementContext;
pub use crate::eml::build::ElementsBuilder;
pub use crate::eml::build::RegisterWidgetExtension;
pub use crate::eml::build::Widget;
pub use crate::eml::build::WidgetBuilder;
pub use crate::eml::content::ExpandElements;
pub use crate::eml::content::ExpandElementsExt;
pub use crate::eml::content::IntoContent;
pub use crate::input::PointerInput;
pub use crate::input::PointerInputData;
pub use crate::property::managed;
pub use crate::relations::Connect;
pub use crate::relations::ConnectionTo;
pub use crate::relations::Signal;

// transformations
pub use crate::relations::bind::Prop;
pub use crate::relations::transform::TransformableTo;

// new bound system
pub use crate::relations::bind::TransformationError;
pub use crate::relations::bind::TransformationResult;
pub use crate::relations::transform::ColorTransformerExtension;

pub use element::Element;
pub use element::Elements;
pub use params::Param;
pub use params::Params;
pub use property::Property;
pub use tagstr;
pub use tagstr::*;
pub use variant::Variant;

use relations::RelationsPlugin;

impl Plugin for ElementsCorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(fix_text_height)
            // .init_resource::<input::Focused>()
            .insert_resource(Defaults::default())
            .add_plugin(ElementsInputPlugin)
            .add_plugin(RelationsPlugin)
            .add_plugin(BuildPligin)
            .add_plugin(EssPlugin)
            .add_plugin(EmlPlugin);

        // TODO: may be desabled with feature
        app.add_startup_system(setup_defaults);

        register_properties(app);
    }
}

fn register_properties(app: &mut bevy::prelude::App) {
    use property::impls::*;

    app.register_property::<DisplayProperty>();
    app.register_property::<PositionTypeProperty>();
    app.register_property::<DirectionProperty>();
    app.register_property::<FlexDirectionProperty>();
    app.register_property::<FlexWrapProperty>();
    app.register_property::<AlignItemsProperty>();
    app.register_property::<AlignSelfProperty>();
    app.register_property::<AlignContentProperty>();
    app.register_property::<JustifyContentProperty>();
    app.register_property::<OverflowProperty>();

    app.register_property::<WidthProperty>();
    app.register_property::<HeightProperty>();
    app.register_property::<MinWidthProperty>();
    app.register_property::<MinHeightProperty>();
    app.register_property::<MaxWidthProperty>();
    app.register_property::<MaxHeightProperty>();
    app.register_property::<FlexBasisProperty>();
    app.register_property::<FlexGrowProperty>();
    app.register_property::<FlexShrinkProperty>();
    app.register_property::<AspectRatioProperty>();

    app.register_compound_property::<PositionProperty>();
    app.register_property::<LeftProperty>();
    app.register_property::<RightProperty>();
    app.register_property::<TopProperty>();
    app.register_property::<BottomProperty>();

    app.register_compound_property::<PaddingProperty>();
    app.register_property::<PaddingLeftProperty>();
    app.register_property::<PaddingRightProperty>();
    app.register_property::<PaddingTopProperty>();
    app.register_property::<PaddingBottomProperty>();

    app.register_compound_property::<MarginProperty>();
    app.register_property::<MarginLeftProperty>();
    app.register_property::<MarginRightProperty>();
    app.register_property::<MarginTopProperty>();
    app.register_property::<MarginBottomProperty>();

    app.register_compound_property::<BorderProperty>();
    app.register_property::<BorderLeftProperty>();
    app.register_property::<BorderRightProperty>();
    app.register_property::<BorderTopProperty>();
    app.register_property::<BorderBottomProperty>();

    app.register_property::<FontColorProperty>();
    app.register_property::<FontProperty>();
    app.register_property::<FontSizeProperty>();
    app.register_property::<VerticalAlignProperty>();
    app.register_property::<HorizontalAlignProperty>();
    app.register_property::<TextContentProperty>();

    app.register_property::<BackgroundColorProperty>();
    app.register_property::<ScaleProperty>();
}

pub struct Widgets;
pub struct Transformers;

#[derive(Bundle)]
pub struct ElementBundle {
    pub element: Element,
    #[bundle]
    pub node: NodeBundle,
}

impl Default for ElementBundle {
    fn default() -> Self {
        ElementBundle {
            element: Default::default(),
            node: NodeBundle {
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct TextElementBundle {
    pub element: Element,
    pub background_color: BackgroundColor,
    #[bundle]
    pub text: TextBundle,
}

impl Default for TextElementBundle {
    fn default() -> Self {
        TextElementBundle {
            element: Element::inline(),
            background_color: BackgroundColor(Color::NONE),
            text: TextBundle {
                text: Text::from_section("", Default::default()),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct ImageElementBundle {
    pub element: Element,
    #[bundle]
    pub image: ImageBundle,
}

impl Default for ImageElementBundle {
    fn default() -> Self {
        ImageElementBundle {
            element: Element::inline(),
            image: ImageBundle {
                background_color: BackgroundColor(Color::WHITE),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ElementsError {
    /// An unsupported selector was found on a style sheet rule.
    UnsupportedSelector,
    /// An unsupported property was found on a style sheet rule.
    UnsupportedProperty(String),
    /// An invalid property value was found on a style sheet rule.
    InvalidPropertyValue(String),
    /// An invalid selector was found on a style sheet rule.
    InvalidSelector,
    /// An unexpected token was found on a style sheet rule.
    UnexpectedToken(String),
}

impl Error for ElementsError {}

impl Display for ElementsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementsError::UnsupportedSelector => {
                write!(f, "Unsupported selector")
            }
            ElementsError::UnsupportedProperty(p) => write!(f, "Unsupported property: {}", p),
            ElementsError::InvalidPropertyValue(p) => write!(f, "Invalid property value: {}", p),
            ElementsError::InvalidSelector => write!(f, "Invalid selector"),
            ElementsError::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
        }
    }
}

pub trait WithElements {
    fn with_elements(&mut self, elements: ElementsBuilder) -> &mut Self;
}

impl<'w, 's, 'a> WithElements for EntityCommands<'w, 's, 'a> {
    fn with_elements(&mut self, elements: ElementsBuilder) -> &mut Self {
        let entity = self.id();
        self.commands().add(elements.with_entity(entity));
        self
    }
}

// pub(crate) type TransformProperty = Box<dyn Fn(&StyleProperty) -> Result<(), ElementsError>>;
pub(crate) type TransformProperty = fn(Variant) -> Result<PropertyValue, ElementsError>;
#[derive(Default, Clone, Resource)]
pub struct PropertyTransformer(Arc<RwLock<HashMap<Tag, TransformProperty>>>);
unsafe impl Send for PropertyTransformer {}
unsafe impl Sync for PropertyTransformer {}
impl PropertyTransformer {
    #[cfg(test)]
    pub(crate) fn new(rules: HashMap<Tag, TransformProperty>) -> PropertyTransformer {
        PropertyTransformer(Arc::new(RwLock::new(rules)))
    }
    pub(crate) fn transform(
        &self,
        name: Tag,
        value: Variant,
    ) -> Result<PropertyValue, ElementsError> {
        self.0
            .read()
            .unwrap()
            .get(&name)
            .ok_or(ElementsError::UnsupportedProperty(name.to_string()))
            .and_then(|transform| transform(value))
    }
}

pub(crate) type ExtractProperty = fn(Variant) -> Result<HashMap<Tag, PropertyValue>, ElementsError>;
#[derive(Default, Clone, Resource)]
pub struct PropertyExtractor(Arc<RwLock<HashMap<Tag, ExtractProperty>>>);
unsafe impl Send for PropertyExtractor {}
unsafe impl Sync for PropertyExtractor {}
impl PropertyExtractor {
    #[cfg(test)]
    pub(crate) fn new(rules: HashMap<Tag, ExtractProperty>) -> PropertyExtractor {
        PropertyExtractor(Arc::new(RwLock::new(rules)))
    }
    pub(crate) fn is_compound_property(&self, name: Tag) -> bool {
        self.0.read().unwrap().contains_key(&name)
    }

    pub(crate) fn extract(
        &self,
        name: Tag,
        value: Variant,
    ) -> Result<HashMap<Tag, PropertyValue>, ElementsError> {
        self.0
            .read()
            .unwrap()
            .get(&name)
            .ok_or(ElementsError::UnsupportedProperty(name.to_string()))
            .and_then(|extractor| extractor(value))
    }
}

pub trait RegisterProperty {
    fn register_property<T: Property + 'static>(&mut self) -> &mut Self;
    fn register_compound_property<T: CompoundProperty + 'static>(&mut self) -> &mut Self;
}

impl RegisterProperty for bevy::prelude::App {
    fn register_property<T: Property + 'static>(&mut self) -> &mut Self {
        self.world
            .get_resource_or_insert_with(PropertyTransformer::default)
            .0
            .write()
            .unwrap()
            .entry(T::name())
            .and_modify(|_| panic!("Property `{}` already registered.", T::name()))
            .or_insert(T::transform);
        self.add_system(T::apply_defaults /* .label(EcssSystem::Apply) */);
        self
    }

    fn register_compound_property<T: CompoundProperty + 'static>(&mut self) -> &mut Self {
        self.world
            .get_resource_or_insert_with(PropertyExtractor::default)
            .0
            .write()
            .unwrap()
            .entry(T::name())
            .and_modify(|_| panic!("CompoundProperty `{}` already registered", T::name()))
            .insert(T::extract);
        self
    }
}

#[derive(Default, Resource)]
pub struct Defaults {
    pub regular_font: Handle<Font>,
    pub italic_font: Handle<Font>,
    pub bold_font: Handle<Font>,
    pub bold_italic_font: Handle<Font>,
    pub style_sheet: Handle<StyleSheet>,
}

pub fn setup_defaults(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    mut defaults: ResMut<Defaults>,
    elements_registry: Res<ElementBuilderRegistry>,
    extractor: Res<PropertyExtractor>,
    validator: Res<PropertyTransformer>,
) {
    let font_bytes = include_bytes!("fonts/Exo2-ExtraLight.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.regular_font = font_handle;
    let font_bytes = include_bytes!("fonts/Exo2-ExtraLightItalic.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.italic_font = font_handle;
    let font_bytes = include_bytes!("fonts/Exo2-SemiBold.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.bold_font = font_handle;
    let font_bytes = include_bytes!("fonts/Exo2-SemiBoldItalic.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.bold_italic_font = font_handle;

    let parser = StyleSheetParser::new(validator.clone(), extractor.clone());
    let mut rules = parser.parse(
        r#"
            * {
                font: regular;
                color: #cfcfcf;
                font-size: 22px;
            }
        "#,
    );
    for rule in elements_registry.styles(parser) {
        rules.push(rule)
    }
    commands.add(StyleSheet::add_default(rules));
}

pub fn fix_text_height(
    mut texts: Query<(&Text, &mut Style), Or<(Changed<Text>, Changed<TextLayoutInfo>)>>,
) {
    for (text, mut style) in texts.iter_mut() {
        if text.sections.len() > 0 {
            style.size.height = Val::Px(text.sections[0].style.font_size);
        }
    }
}
