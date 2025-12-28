use bitflags::bitflags;

bitflags! {
    pub struct RuinStrategy: u32 {
        const Rect = 1 << 0;
        const Annotation = 1 << 1;
        const Image = 1 << 2;
    }
}
