use bitflags::bitflags;

bitflags! {
    pub struct RuinStrategy: u32 {
        const Rect = 1 << 0;
        const Reserved1 = 1 << 1;
        const Reserved2 = 1 << 2;
        const Reserved3 = 1 << 3;
        const Image = 1 << 4;
    }
}
