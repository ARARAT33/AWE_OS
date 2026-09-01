#![no_std]

use crate::runtime::InputEvent;

pub const MAX_WINDOWS: usize = 32;
pub const MAX_WIDTH: u32 = 8192;
pub const MAX_HEIGHT: u32 = 8192;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect { pub x: i32, pub y: i32, pub width: u32, pub height: u32 }
impl Rect {
    pub const fn valid(self) -> bool { self.width > 0 && self.height > 0 && self.width <= MAX_WIDTH && self.height <= MAX_HEIGHT }
    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x.saturating_add(self.width as i32) && y < self.y.saturating_add(self.height as i32)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Window { pub id: u16, pub rect: Rect, pub visible: bool, pub focused: bool, pub z: u16 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowError { Full, Invalid, NotFound, BufferTooSmall, InvalidFramebuffer, Overflow }

pub struct WindowManager { windows: [Option<Window>; MAX_WINDOWS], count: usize, next_id: u16, focused: Option<u16> }
impl WindowManager {
    pub const fn new() -> Self { Self { windows: [None; MAX_WINDOWS], count: 0, next_id: 1, focused: None } }
    pub const fn count(&self) -> usize { self.count }
    pub const fn focused(&self) -> Option<u16> { self.focused }

    pub fn create(&mut self, rect: Rect) -> Result<u16, WindowError> {
        if !rect.valid() { return Err(WindowError::Invalid); }
        let slot = self.windows.iter().position(Option::is_none).ok_or(WindowError::Full)?;
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let z = self.count as u16;
        self.windows[slot] = Some(Window { id, rect, visible: true, focused: false, z });
        self.count += 1;
        self.focus(id)?;
        Ok(id)
    }

    pub fn destroy(&mut self, id: u16) -> Result<(), WindowError> {
        let index = self.find(id).ok_or(WindowError::NotFound)?;
        self.windows[index] = None;
        self.count -= 1;
        if self.focused == Some(id) { self.focused = None; self.raise_top_focus(); }
        Ok(())
    }

    pub fn focus(&mut self, id: u16) -> Result<(), WindowError> {
        let index = self.find(id).ok_or(WindowError::NotFound)?;
        for window in self.windows.iter_mut().flatten() { window.focused = false; }
        self.windows[index].as_mut().unwrap().focused = true;
        self.focused = Some(id);
        let max_z = self.windows.iter().flatten().map(|w| w.z).max().unwrap_or(0);
        self.windows[index].as_mut().unwrap().z = max_z.saturating_add(1);
        Ok(())
    }

    pub fn move_window(&mut self, id: u16, x: i32, y: i32) -> Result<(), WindowError> {
        let index = self.find(id).ok_or(WindowError::NotFound)?;
        self.windows[index].as_mut().unwrap().rect.x = x;
        self.windows[index].as_mut().unwrap().rect.y = y;
        Ok(())
    }

    pub fn resize(&mut self, id: u16, width: u32, height: u32) -> Result<(), WindowError> {
        let index = self.find(id).ok_or(WindowError::NotFound)?;
        let rect = Rect { x: self.windows[index].unwrap().rect.x, y: self.windows[index].unwrap().rect.y, width, height };
        if !rect.valid() { return Err(WindowError::Invalid); }
        self.windows[index].as_mut().unwrap().rect = rect;
        Ok(())
    }

    pub fn hit_test(&mut self, x: i32, y: i32) -> Option<u16> {
        let mut best: Option<Window> = None;
        for window in self.windows.iter().flatten() {
            if window.visible && window.rect.contains(x, y) && best.map(|b| window.z > b.z).unwrap_or(true) { best = Some(*window); }
        }
        if let Some(w) = best { let _ = self.focus(w.id); Some(w.id) } else { None }
    }

    pub fn handle_input(&mut self, event: InputEvent) -> Option<u16> {
        match event { InputEvent::Pointer { x, y, .. } => self.hit_test(x, y), InputEvent::Key { .. } => self.focused }
    }

    fn find(&self, id: u16) -> Option<usize> { self.windows.iter().position(|w| w.map(|v| v.id) == Some(id)) }
    fn raise_top_focus(&mut self) { if let Some(id) = self.windows.iter().flatten().max_by_key(|w| w.z).map(|w| w.id) { let _ = self.focus(id); } }
}
impl Default for WindowManager { fn default() -> Self { Self::new() } }

pub struct DoubleBuffer<'a> { pub width: u32, pub height: u32, pub pitch: u32, pub front: &'a mut [u8], pub back: &'a mut [u8] }
impl<'a> DoubleBuffer<'a> {
    pub fn new(width: u32, height: u32, pitch: u32, front: &'a mut [u8], back: &'a mut [u8]) -> Result<Self, WindowError> {
        if width == 0 || height == 0 || pitch < width.saturating_mul(4) { return Err(WindowError::InvalidFramebuffer); }
        let bytes = (height as usize).checked_mul(pitch as usize).ok_or(WindowError::Overflow)?;
        if front.len() < bytes || back.len() < bytes { return Err(WindowError::BufferTooSmall); }
        Ok(Self { width, height, pitch, front, back })
    }
    pub fn clear(&mut self, pixel: [u8; 4]) {
        for y in 0..self.height as usize {
            let row = y * self.pitch as usize;
            for x in 0..self.width as usize { let o = row + x * 4; self.back[o..o+4].copy_from_slice(&pixel); }
        }
    }
    pub fn fill_rect(&mut self, rect: Rect, pixel: [u8; 4]) {
        let x0 = rect.x.max(0) as u32; let y0 = rect.y.max(0) as u32;
        let x1 = rect.x.saturating_add(rect.width as i32).max(0) as u32;
        let y1 = rect.y.saturating_add(rect.height as i32).max(0) as u32;
        let x1 = x1.min(self.width); let y1 = y1.min(self.height);
        for y in y0.min(self.height)..y1 { let row = y as usize * self.pitch as usize; for x in x0.min(self.width)..x1 { let o = row + x as usize * 4; self.back[o..o+4].copy_from_slice(&pixel); } }
    }
    pub fn present(&mut self) { self.front[..self.back.len()].copy_from_slice(self.back); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn window_manager_focus_hit_move_resize_and_destroy() {
        let mut wm = WindowManager::new();
        let a = wm.create(Rect { x: 10, y: 10, width: 100, height: 100 }).unwrap();
        let b = wm.create(Rect { x: 20, y: 20, width: 100, height: 100 }).unwrap();
        assert_eq!(wm.hit_test(30, 30), Some(b));
        wm.move_window(b, 200, 200).unwrap();
        wm.resize(a, 120, 120).unwrap();
        assert_eq!(wm.hit_test(30, 30), Some(a));
        wm.destroy(a).unwrap();
        assert_eq!(wm.count(), 1);
    }

    #[test]
    fn double_buffer_validates_stride_and_capacity() {
        let mut front = [0u8; 64]; let mut back = [0u8; 64];
        let mut db = DoubleBuffer::new(4, 4, 16, &mut front, &mut back).unwrap();
        db.clear([1, 2, 3, 4]);
        db.fill_rect(Rect { x: 1, y: 1, width: 2, height: 2 }, [4, 3, 2, 1]);
        db.present();
        assert_eq!(front[0..4], [1,2,3,4]);
    }
}