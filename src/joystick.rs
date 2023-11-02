use std::hash::Hash;

use bevy::{
    prelude::*,
    render::Extract,
    ui::{
        ContentSize, ExtractedUiNode, ExtractedUiNodes, FocusPolicy, RelativeCursorPosition,
        UiStack,
    },
};

use crate::TouchStickType;

/// The tint color of the image
///
/// When combined with [`VirtualJoystickNode`], tints the provided texture, while still
/// respecting transparent areas.
#[derive(Component, Copy, Clone, Debug, Reflect)]
#[reflect(Component, Default)]
pub struct TintColor(pub Color);

impl TintColor {
    pub const DEFAULT: Self = Self(Color::WHITE);
}

impl Default for TintColor {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<Color> for TintColor {
    fn from(color: Color) -> Self {
        Self(color)
    }
}

/// Marker component for a bevy_ui Node area where sticks can be interacted with.
#[derive(Component, Copy, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct TouchStickInteractionArea;

// TODO: default returns a broken bundle, should remove or fix
#[derive(Bundle, Debug, Default)]
pub struct TouchStickBundle<
    S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + 'static,
> {
    /// Describes the size of the node
    pub(crate) node: Node,
    /// Describes the style including flexbox settings
    pub(crate) style: Style,
    /// The calculated size based on the given image
    pub(crate) calculated_size: ContentSize,
    /// The tint color of the image
    pub(crate) color: TintColor,
    /// The texture atlas image of the node
    pub(crate) joystick: TouchStickNode<S>,
    /// Whether this node should block interaction with lower nodes
    pub(crate) focus_policy: FocusPolicy,
    /// The transform of the node
    pub(crate) transform: Transform,
    /// The global transform of the node
    pub(crate) global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub(crate) visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub(crate) computed_visibility: ComputedVisibility,
    /// Indicates the depth at which the node should appear in the UI
    pub(crate) z_index: ZIndex,
    pub(crate) knob_data: TouchStickKnob,
    pub(crate) cursor_pos: RelativeCursorPosition,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct TouchStickNode<S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + 'static>
{
    /// Identifier of joystick
    pub id: S,
    /// Image for background or border image on joystick
    pub border_image: Handle<Image>,
    /// Image for handler knob on joystick
    pub knob_image: Handle<Image>,
    /// Size for knob on joystick
    pub knob_size: Vec2,
    /// Zone to ignore movement
    pub dead_zone: f32,
    /// Define the behaviour of joystick
    pub behavior: TouchStickType,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct TouchStickKnob {
    pub drag_id: Option<u64>,
    pub dead_zone: f32,
    pub base_position: Vec2,
    pub start_position: Vec2,
    pub current_position: Vec2,
    /// Value with maximum magnitude 1
    pub value: Vec2,
    pub interactable_zone: Rect,
}

impl<S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + 'static>
    TouchStickBundle<S>
{
    pub fn new(joystick: TouchStickNode<S>) -> Self {
        Self {
            joystick,
            ..default()
        }
    }

    pub fn set_node(mut self, node: Node) -> Self {
        self.node = node;
        self
    }

    pub fn set_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn set_color(mut self, color: TintColor) -> Self {
        self.color = color;
        self
    }

    pub fn set_focus_policy(mut self, focus_policy: FocusPolicy) -> Self {
        self.focus_policy = focus_policy;
        self
    }

    pub fn set_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn set_global_transform(mut self, global_transform: GlobalTransform) -> Self {
        self.global_transform = global_transform;
        self
    }

    pub fn set_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn set_computed_visibility(mut self, computed_visibility: ComputedVisibility) -> Self {
        self.computed_visibility = computed_visibility;
        self
    }

    pub fn set_z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = z_index;
        self
    }
}

#[allow(clippy::type_complexity)]
pub fn extract_joystick_node<
    S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + 'static,
>(
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    images: Extract<Res<Assets<Image>>>,
    ui_stack: Extract<Res<UiStack>>,
    uinode_query: Extract<
        Query<(
            &Node,
            &GlobalTransform,
            &TintColor,
            &TouchStickNode<S>,
            &ComputedVisibility,
            &TouchStickKnob,
        )>,
    >,
) {
    for (stack_index, entity) in ui_stack.uinodes.iter().enumerate() {
        if let Ok((uinode, global_transform, color, joystick_node, visibility, data)) =
            uinode_query.get(*entity)
        {
            if !visibility.is_visible()
                || uinode.size().x == 0.
                || uinode.size().y == 0.
                || color.0.a() == 0.0
                || !images.contains(&joystick_node.border_image)
                || !images.contains(&joystick_node.knob_image)
                || data.drag_id.is_none() && joystick_node.behavior == TouchStickType::Dynamic
            {
                continue;
            }
            let container_rect = Rect {
                max: uinode.size(),
                ..default()
            };

            let border_pos = match joystick_node.behavior {
                TouchStickType::Fixed => global_transform
                    .compute_matrix()
                    .transform_point3((container_rect.center() - (uinode.size() / 2.)).extend(0.)),
                TouchStickType::Floating => {
                    if data.drag_id.is_none() {
                        global_transform.compute_matrix().transform_point3(
                            (container_rect.center() - (uinode.size() / 2.)).extend(0.),
                        )
                    } else {
                        data.start_position.extend(0.)
                    }
                }
                TouchStickType::Dynamic => data.base_position.extend(0.),
            };

            extracted_uinodes.uinodes.push(ExtractedUiNode {
                stack_index,
                transform: Mat4::from_translation(border_pos),
                color: color.0,
                rect: container_rect,
                image: joystick_node.border_image.clone(),
                atlas_size: None,
                clip: None,
                flip_x: false,
                flip_y: false,
            });

            let rect = Rect {
                max: joystick_node.knob_size,
                ..default()
            };

            let radius = uinode.size().x / 2.;
            let axis_value = data.value;
            // ui is y down, so we flip
            let pos = Vec2::new(axis_value.x, -axis_value.y) * radius;

            let knob_pos = match joystick_node.behavior {
                TouchStickType::Fixed => global_transform.compute_matrix().transform_point3(
                    (container_rect.center() - (uinode.size() / 2.) + pos).extend(0.),
                ),
                TouchStickType::Floating => {
                    if data.drag_id.is_none() {
                        global_transform.compute_matrix().transform_point3(
                            (container_rect.center() - (uinode.size() / 2.)).extend(0.),
                        )
                    } else {
                        (data.start_position + pos).extend(0.)
                    }
                }
                TouchStickType::Dynamic => (data.base_position + pos).extend(0.),
            };

            extracted_uinodes.uinodes.push(ExtractedUiNode {
                rect,
                stack_index,
                transform: Mat4::from_translation(knob_pos),
                color: color.0,
                image: joystick_node.knob_image.clone(),
                atlas_size: None,
                clip: None,
                flip_x: false,
                flip_y: false,
            });
        }
    }
}
