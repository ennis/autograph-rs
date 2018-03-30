use nalgebra as na;

pub struct RectTransform
{
    /// rect corners relative to anchors
    pub offset_a: na::Point2<f32>,
    pub offset_b: na::Point2<f32>,
    /// upper-left anchor rect position (percentage)
    pub anchor_a: na::Point2<f32>,
    /// lower-right anchor rect position (percentage)
    pub anchor_b: na::Point2<f32>,
    /// pivot position relative to anchored rectangle
    pub pivot: na::Vector2<f32>,
    /// rotation amount around pivot
    pub rotation: na::Rotation2<f32>,
    /// scale around pivot
    pub scale: na::Vector2<f32>
}

impl Default for RectTransform
{
    fn default() -> Self {
        RectTransform {
            offset_a: na::Point2::new(0.0,0.0),
            offset_b: na::Point2::new(0.0,0.0),
            anchor_a: na::Point2::new(0.0,0.0),
            anchor_b: na::Point2::new(1.0,1.0),
            pivot: na::Vector2::new(0.5,0.5),
            rotation: na::Rotation2::new(0.0),
            scale: na::Vector2::new(1.0,1.0),
        }
    }
}

pub struct CalculatedRectTransform
{
    pub transform: na::Matrix3<f32>,
    pub size: na::Vector2<f32>
}

#[derive(Copy,Clone,Debug)]
pub enum HorizontalAnchor
{
    /// Maintain a constant size and anchor to the left border with a constant pixel offset w.r.t. the border
    Left { offset: f32, size: u32 },
    /// Maintain a constant size and anchor to the center line with a constant pixel offset w.r.t. the line
    Center { offset: f32, size: u32 },
    /// Maintain a constant size and anchor to the right border with a constant pixel offset w.r.t. the border
    Right { offset: f32, size: u32 },
    ///
    Proportional { prop: f32, offset: f32, size: u32 },
    /// Stretch box and maintain a constant pixel inset
    Stretch { left_prop: f32, right_prop: f32, left_inset: f32, right_inset: f32 },
}

#[derive(Copy,Clone,Debug)]
pub enum VerticalAnchor
{
    /// Maintain a constant size and anchor to the left border with a constant pixel offset w.r.t. the border
    Bottom { offset: f32, size: u32 },
    /// Maintain a constant size and anchor to the center line with a constant pixel offset w.r.t. the line
    Center { offset: f32, size: u32 },
    /// Maintain a constant size and anchor to the right border with a constant pixel offset w.r.t. the border
    Top { offset: f32, size: u32 },
    ///
    Proportional { prop: f32, offset: f32, size: u32 },
    /// Stretch box with custom proportions and maintain a constant pixel inset
    Stretch { bottom_prop: f32, top_prop: f32, bottom_inset: f32, top_inset: f32 },
}

impl RectTransform
{
    /// parent_size: parent size in pixels for pixel anchors
    pub fn calculate_in_parent(&self, parent_transform: &na::Matrix3<f32>, parent_size: &na::Vector2<f32>) -> CalculatedRectTransform {
        let par_w = parent_size.x;
        let par_h = parent_size.y;
        let par_aspect = par_w / par_h;
        let left = f32::round(self.offset_a.x);
        let right = f32::round(self.offset_b.x);
        let top = f32::round(self.offset_b.y);
        let bottom = f32::round(self.offset_a.y);
        let a = self.anchor_a.x + left / par_w;
        let b = self.anchor_b.x + right / par_w;
        let c = self.anchor_a.y + bottom / par_h;
        let d = self.anchor_b.y + top / par_h;

        use alga::linear::Transformation;
        // target space for rotation must be correct w.r.t aspect ratio of parent
        let anchor_transform = na::Matrix3::new(par_aspect*(b-a), 0.0, a, 0.0, d-c, c, 0.0, 0.0, 1.0);
        let pivot = anchor_transform.transform_point(&na::Point::from_coordinates(self.pivot));
        let scale_rot = na::Matrix3::new_translation(&pivot.coords) *
            na::Matrix3::new_nonuniform_scaling(&self.scale) *
            self.rotation.to_homogeneous() *
            na::Matrix3::new_translation(&-pivot.coords);
        // result in texture space (0,1)x(0,1)
        let final_transform =
            na::Matrix3::new_nonuniform_scaling(&na::Vector2::new(1.0/par_aspect,1.0)) *
                scale_rot *
                anchor_transform;

        CalculatedRectTransform {
            transform: final_transform,
            size: *parent_size   // TODO
        }
    }

    pub fn new(horizontal_anchor: HorizontalAnchor, vertical_anchor: VerticalAnchor) -> RectTransform
    {
        let (a,b,left,right) = match horizontal_anchor {
            HorizontalAnchor::Left{ offset, size } => { (0.0, 0.0, offset, offset + size as f32) },
            HorizontalAnchor::Center{ offset, size } => { (0.5, 0.5, offset - size as f32 / 2.0, offset + size as f32 / 2.0) },
            HorizontalAnchor::Right{ offset, size } => { (1.0, 1.0, -(offset + size as f32), -offset) },
            HorizontalAnchor::Proportional { prop, offset, size } => { (prop, prop, offset - size as f32 / 2.0, offset + size as f32 / 2.0) },
            HorizontalAnchor::Stretch{ left_prop, right_prop, left_inset, right_inset } => { (left_prop, right_prop, left_inset, -right_inset) },
        };

        let (c,d,bottom,top) = match vertical_anchor {
            VerticalAnchor::Bottom{ offset, size } => { (0.0, 0.0, offset, offset + size as f32) },
            VerticalAnchor::Center{ offset, size } => { (0.5, 0.5, offset - size as f32 / 2.0, offset + size as f32 / 2.0) },
            VerticalAnchor::Top{ offset, size } => { (1.0, 1.0, -(offset + size as f32), -offset) },
            VerticalAnchor::Proportional { prop, offset, size } => { (prop, prop, offset - size as f32 / 2.0, offset + size as f32 / 2.0) },
            VerticalAnchor::Stretch{ bottom_prop, top_prop, bottom_inset, top_inset } => { (bottom_prop, top_prop, bottom_inset, -top_inset) },
        };

        //debug!("(a,b,left,right)={},{},{},{}", a,b,left,right);
        //debug!("(c,d,bottom,top)={},{},{},{}", c,d,bottom,top);

        RectTransform {
            offset_a: na::Point2::new(left,bottom),
            offset_b: na::Point2::new(right,top),
            anchor_a: na::Point2::new(a,c),
            anchor_b: na::Point2::new(b,d),
            pivot: na::Vector2::new(0.5,0.5),
            rotation: na::Rotation2::new(0.0),
            scale: na::Vector2::new(1.0,1.0),
        }
    }

    pub fn with_rotation(self, rotation: f32) -> Self {
        RectTransform {
            rotation: na::Rotation2::new(rotation),
            .. self
        }
    }

    pub fn with_scale(self, scale: f32) -> Self {
        RectTransform {
            scale: na::Vector2::new(scale,scale),
            ..self
        }
    }
}
