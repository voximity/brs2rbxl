use std::{
    collections::{hash_map::Entry, HashMap},
    f32::consts::PI,
};

use brickadia::save::{Brick, BrickColor, Color, SaveData, Size, UnrealType};
use rbx_dom_weak::{
    types::{CFrame, Color3, Enum, Variant, Vector3},
    InstanceBuilder,
};

use crate::cframe::CoordinateFrame;

macro_rules! rm {
    (
        r($rx:literal, $ry:literal, $rz:literal),
        u($ux:literal, $uy:literal, $uz:literal),
        f($fx:literal, $fy:literal, $fz:literal)
    ) => {
        [$rx, $ux, -$fx, $ry, $uy, -$fy, $rz, $uz, -$fz]
    };
}

macro_rules! component_property {
    ($component:ident, $field:expr, $variant:path, $default:expr) => {
        if let Some($variant(_inner)) = $component.get($field) {
            _inner
        } else {
            $default
        }
    };
}

static ORIENTATION_MAP: [[f32; 9]; 24] = [
    rm!(r(0.0, -1.0, 0.0), u(1.0, 0.0, 0.0), f(0.0, 0.0, -1.0)),
    rm!(r(0.0, 0.0, 1.0), u(1.0, 0.0, 0.0), f(0.0, -1.0, 0.0)),
    rm!(r(0.0, 1.0, 0.0), u(1.0, 0.0, 0.0), f(0.0, 0.0, 1.0)),
    rm!(r(0.0, 0.0, -1.0), u(1.0, 0.0, 0.0), f(0.0, 1.0, 0.0)),
    rm!(r(0.0, -1.0, 0.0), u(-1.0, 0.0, 0.0), f(0.0, 0.0, 1.0)),
    rm!(r(0.0, 0.0, -1.0), u(-1.0, 0.0, 0.0), f(0.0, -1.0, 0.0)),
    rm!(r(0.0, 1.0, 0.0), u(-1.0, 0.0, 0.0), f(0.0, 0.0, -1.0)),
    rm!(r(0.0, 0.0, 1.0), u(-1.0, 0.0, 0.0), f(0.0, 1.0, 0.0)),
    rm!(r(0.0, -1.0, 0.0), u(0.0, 0.0, 1.0), f(1.0, 0.0, 0.0)),
    rm!(r(-1.0, 0.0, 0.0), u(0.0, 0.0, 1.0), f(0.0, -1.0, 0.0)),
    rm!(r(0.0, 1.0, 0.0), u(0.0, 0.0, 1.0), f(-1.0, 0.0, 0.0)),
    rm!(r(1.0, 0.0, 0.0), u(0.0, 0.0, 1.0), f(0.0, 1.0, 0.0)),
    rm!(r(0.0, -1.0, 0.0), u(0.0, 0.0, -1.0), f(-1.0, 0.0, 0.0)),
    rm!(r(1.0, 0.0, 0.0), u(0.0, 0.0, -1.0), f(0.0, -1.0, 0.0)),
    rm!(r(0.0, 1.0, 0.0), u(0.0, 0.0, -1.0), f(1.0, 0.0, 0.0)),
    rm!(r(-1.0, 0.0, 0.0), u(0.0, 0.0, -1.0), f(0.0, 1.0, 0.0)),
    rm!(r(-1.0, 0.0, 0.0), u(0.0, 1.0, 0.0), f(0.0, 0.0, 1.0)),
    rm!(r(0.0, 0.0, -1.0), u(0.0, 1.0, 0.0), f(-1.0, 0.0, 0.0)),
    rm!(r(1.0, 0.0, 0.0), u(0.0, 1.0, 0.0), f(0.0, 0.0, -1.0)),
    rm!(r(0.0, 0.0, 1.0), u(0.0, 1.0, 0.0), f(1.0, 0.0, 0.0)),
    rm!(r(1.0, 0.0, 0.0), u(0.0, -1.0, 0.0), f(0.0, 0.0, 1.0)),
    rm!(r(0.0, 0.0, -1.0), u(0.0, -1.0, 0.0), f(1.0, 0.0, 0.0)),
    rm!(r(-1.0, 0.0, 0.0), u(0.0, -1.0, 0.0), f(0.0, 0.0, -1.0)),
    rm!(r(0.0, 0.0, 1.0), u(0.0, -1.0, 0.0), f(-1.0, 0.0, 0.0)),
];

pub struct PartDef {
    class: String,
    offset: CoordinateFrame,
    size: Vector3,
    color: Option<Color>,
    properties: HashMap<String, Variant>,
}

impl Default for PartDef {
    fn default() -> Self {
        Self {
            class: "Part".into(),
            offset: CoordinateFrame::default(),
            size: Vector3::new(0.0, 0.0, 0.0),
            color: None,
            properties: HashMap::new(),
        }
    }
}

#[allow(dead_code)]
impl PartDef {
    pub fn new(class: impl Into<String>) -> Self {
        Self {
            class: class.into(),
            ..Default::default()
        }
    }

    pub fn cf(mut self, cf: CoordinateFrame) -> Self {
        self.offset = self.offset * cf;
        self
    }

    pub fn offset(self, x: f32, y: f32, z: f32) -> Self {
        self.cf(CoordinateFrame::new(x, y, z))
    }

    pub fn size(mut self, x: f32, y: f32, z: f32) -> Self {
        self.size = Vector3::new(x, y, z);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn property<K: Into<String>, V: Into<Variant>>(mut self, key: K, value: V) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub fn to_instance(self, save: &SaveData, brick: &Brick) -> InstanceBuilder {
        let mut instance = InstanceBuilder::new(self.class);

        // write size
        instance.add_property("Size", self.size);

        // write cframe
        let mat_comp =
            ORIENTATION_MAP[((brick.direction as u8) << 2 | (brick.rotation as u8)) as usize];

        instance.add_property(
            "CFrame",
            CFrame::from(
                CoordinateFrame::from_rotation(
                    brick.position.0 as f32 / 10.0,
                    brick.position.2 as f32 / 10.0,
                    brick.position.1 as f32 / 10.0,
                    mat_comp,
                ) * self.offset,
            ),
        );

        // write color
        let color = self.color.as_ref().unwrap_or_else(|| match &brick.color {
            BrickColor::Index(idx) => &save.header2.colors[*idx as usize],
            BrickColor::Unique(c) => c,
        });

        let color_value = Color3::new(
            linear_to_srgb(color.r as f32 / 255.0),
            linear_to_srgb(color.g as f32 / 255.0),
            linear_to_srgb(color.b as f32 / 255.0),
        );
        instance.add_property("Color", color_value);

        // write material
        if brick.visibility {
            match save.header2.materials[brick.material_index as usize].as_str() {
                "BMC_Ghost" | "BMC_Ghost_Fail" => {
                    instance.add_property("Material", Enum::from_u32(288));
                    instance.add_property("Transparency", 0.5);
                }
                "BMC_Glow" => instance.add_property("Material", Enum::from_u32(288)),
                "BMC_Metallic" => instance.add_property("Material", Enum::from_u32(1088)),
                "BMC_Hologram" => instance.add_property("Material", Enum::from_u32(1584)),
                "BMC_Glass" => instance.add_property(
                    "Transparency",
                    1.0 - (brick.material_intensity as f32 / 10.0),
                ),
                _ => (),
            }
        } else {
            instance.add_property("Transparency", 1.0f32);
        }

        // collision
        if !brick.collision.player {
            instance.add_property("CanCollide", false);
        }

        // anchor
        instance.add_property("Anchored", true);

        // components
        match brick.components.get("BCD_PointLight") {
            Some(component) => {
                let mut light = InstanceBuilder::new("PointLight");
                light.add_property(
                    "Brightness",
                    component_property!(component, "Brightness", UnrealType::Float, &10.0) / 10.0,
                );
                light.add_property(
                    "Range",
                    component_property!(component, "Range", UnrealType::Float, &100.0) / 10.0,
                );
                light.add_property(
                    "Shadows",
                    *component_property!(component, "bCastShadows", UnrealType::Boolean, &false),
                );
                if let UnrealType::Boolean(true) = component["bUseBrickColor"] {
                    light.add_property("Color", color_value);
                } else {
                    let color = component_property!(
                        component,
                        "Color",
                        UnrealType::Color,
                        &Color {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 255
                        }
                    );

                    light.add_property(
                        "Color",
                        Color3::new(
                            linear_to_srgb(color.r as f32 / 255.0),
                            linear_to_srgb(color.g as f32 / 255.0),
                            linear_to_srgb(color.b as f32 / 255.0),
                        ),
                    );
                }
                instance.add_child(light);
            }
            _ => (),
        }

        self.properties
            .into_iter()
            .for_each(|(key, value)| instance.add_property(key, value));

        instance
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c > 0.0031308 {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * c
    }
}

pub fn convert_brick(brick: &Brick, save: &SaveData) -> Option<Vec<InstanceBuilder>> {
    let asset = save.header2.brick_assets[brick.asset_name_index as usize].as_str();

    let size = match brick.size {
        Size::Empty => (0.0, 0.0, 0.0),
        Size::Procedural(x, y, z) => (x as f32 / 5.0, y as f32 / 5.0, z as f32 / 5.0),
    };

    return Some(match asset {
        "PB_DefaultBrick" => vec![PartDef::default()
            .size(size.0, size.2, size.1)
            .to_instance(&save, brick)],

        "PB_DefaultTile" => vec![PartDef::default()
            .size(size.0, size.2, size.1)
            .property("TopSurface", Enum::from_u32(0))
            .to_instance(&save, brick)],

        "PB_DefaultRamp" => vec![
            PartDef::new("Part")
                .size(1.0, size.2, size.1)
                .offset(size.0 / 2.0 - 0.5, 0.0, 0.0)
                .to_instance(&save, brick),
            PartDef::new("WedgePart")
                .size(size.1, size.2 - 0.2, size.0 - 1.0)
                .offset(-0.5, 0.1, 0.0)
                .cf(CoordinateFrame::ry(PI * 0.5))
                .to_instance(&save, brick),
            PartDef::new("Part")
                .size(size.0 - 1.0, 0.2, size.1)
                .offset(-0.5, -(size.2 / 2.0) + 0.1, 0.0)
                .to_instance(&save, brick),
        ],

        "PB_DefaultRampInverted" => vec![
            PartDef::new("Part")
                .size(1.0, size.2, size.1)
                .offset(size.0 / 2.0 - 0.5, 0.0, 0.0)
                .to_instance(&save, brick),
            PartDef::new("WedgePart")
                .size(size.1, size.2 - 0.2, size.0 - 1.0)
                .offset(-0.5, -0.1, 0.0)
                .cf(CoordinateFrame::rx(PI))
                .cf(CoordinateFrame::ry(PI * 0.5))
                .to_instance(&save, brick),
            PartDef::new("Part")
                .size(size.0 - 1.0, 0.2, size.1)
                .offset(-0.5, (size.2 / 2.0) - 0.1, 0.0)
                .to_instance(&save, brick),
        ],

        "PB_DefaultWedge" => vec![
            PartDef::new("WedgePart")
                .size(size.1, size.2 - 0.2, size.0)
                .offset(0.0, 0.1, 0.0)
                .cf(CoordinateFrame::ry(PI * 0.5))
                .to_instance(&save, brick),
            PartDef::new("Part")
                .size(size.1, 0.2, size.0)
                .offset(0.0, -(size.2 / 2.0) + 0.1, 0.0)
                .cf(CoordinateFrame::ry(PI * 0.5))
                .to_instance(&save, brick),
        ],

        "PB_DefaultSideWedge" => vec![PartDef::new("WedgePart")
            .size(size.2, size.0, size.1)
            .cf(CoordinateFrame::rz(PI * 0.5))
            .property("TopSurface", Enum::from_u32(0))
            .property("BottomSurface", Enum::from_u32(0))
            .property("LeftSurface", Enum::from_u32(4))
            .property("RightSurface", Enum::from_u32(3))
            .to_instance(&save, brick)],

        "PB_DefaultSideWedgeTile" => vec![PartDef::new("WedgePart")
            .size(size.2, size.0, size.1)
            .cf(CoordinateFrame::rz(PI * 0.5))
            .property("TopSurface", Enum::from_u32(0))
            .property("BottomSurface", Enum::from_u32(0))
            .property("LeftSurface", Enum::from_u32(4))
            .property("RightSurface", Enum::from_u32(0))
            .to_instance(&save, brick)],

        "PB_DefaultMicroBrick" => vec![PartDef::new("Part")
            .size(size.0, size.2, size.1)
            .property("TopSurface", Enum::from_u32(0))
            .property("BottomSurface", Enum::from_u32(0))
            .to_instance(&save, brick)],

        "PB_DefaultMicroWedge" => vec![PartDef::new("WedgePart")
            .size(size.2, size.1, size.0)
            .cf(CoordinateFrame::rz(PI * 0.5))
            .cf(CoordinateFrame::rx(-PI * 0.5))
            .cf(CoordinateFrame::ry(PI))
            .property("BottomSurface", Enum::from_u32(0))
            .to_instance(&save, brick)],

        "PB_DefaultMicroWedgeInnerCorner" => vec![
            PartDef::new("WedgePart")
                .size(size.0, size.2, size.1)
                .cf(CoordinateFrame::ry(PI * 0.5))
                .property("BottomSurface", Enum::from_u32(0))
                .to_instance(&save, brick),
            PartDef::new("WedgePart")
                .size(size.1, size.2, size.0)
                .property("BottomSurface", Enum::from_u32(0))
                .to_instance(&save, brick),
        ],

        "B_2x2_Round" => vec![PartDef::new("Part")
            .size(1.2, 2.0, 2.0)
            .cf(CoordinateFrame::rz(PI * 0.5))
            .property("Shape", Enum::from_u32(2))
            .property("TopSurface", Enum::from_u32(0))
            .property("BottomSurface", Enum::from_u32(0))
            .property("LeftSurface", Enum::from_u32(4))
            .property("RightSurface", Enum::from_u32(3))
            .to_instance(&save, brick)],

        "B_2x2F_Round" => vec![PartDef::new("Part")
            .size(0.4, 2.0, 2.0)
            .cf(CoordinateFrame::rz(PI * 0.5))
            .property("Shape", Enum::from_u32(2))
            .property("TopSurface", Enum::from_u32(0))
            .property("BottomSurface", Enum::from_u32(0))
            .property("LeftSurface", Enum::from_u32(4))
            .property("RightSurface", Enum::from_u32(3))
            .to_instance(&save, brick)],

        "B_1x1_Round" | "B_1x1_Cone" => vec![PartDef::new("Part")
            .size(1.2, 1.0, 1.0)
            .cf(CoordinateFrame::rz(PI * 0.5))
            .property("Shape", Enum::from_u32(2))
            .property("TopSurface", Enum::from_u32(0))
            .property("BottomSurface", Enum::from_u32(0))
            .property("LeftSurface", Enum::from_u32(4))
            .property("RightSurface", Enum::from_u32(3))
            .to_instance(&save, brick)],

        "B_1x1F_Round" => vec![PartDef::new("Part")
            .size(1.2, 1.0, 1.0)
            .cf(CoordinateFrame::rz(PI * 0.5))
            .property("Shape", Enum::from_u32(2))
            .property("TopSurface", Enum::from_u32(0))
            .property("BottomSurface", Enum::from_u32(0))
            .property("LeftSurface", Enum::from_u32(4))
            .property("RightSurface", Enum::from_u32(3))
            .to_instance(&save, brick)],

        _ => return None,
    });
}
