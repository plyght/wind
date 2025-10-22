use std::ops::Range;

pub struct LazyList<T> {
    items: Vec<T>,
    viewport_size: usize,
    viewport_offset: usize,
    selected_index: usize,
    page_size: usize,
}

impl<T: Clone> LazyList<T> {
    pub fn new(page_size: usize) -> Self {
        Self {
            items: Vec::new(),
            viewport_size: 20,
            viewport_offset: 0,
            selected_index: 0,
            page_size,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.selected_index = self.selected_index.min(self.items.len().saturating_sub(1));
    }

    pub fn set_viewport_size(&mut self, size: usize) {
        self.viewport_size = size;
    }

    pub fn visible_range(&self) -> Range<usize> {
        let start = self.viewport_offset;
        let end = (self.viewport_offset + self.viewport_size).min(self.items.len());
        start..end
    }

    pub fn visible_items(&self) -> &[T] {
        let range = self.visible_range();
        &self.items[range]
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }

        let new_index = (self.selected_index as i32 + delta)
            .max(0)
            .min(self.items.len() as i32 - 1) as usize;

        self.selected_index = new_index;

        if self.selected_index < self.viewport_offset {
            self.viewport_offset = self.selected_index;
        } else if self.selected_index >= self.viewport_offset + self.viewport_size {
            self.viewport_offset = self.selected_index - self.viewport_size + 1;
        }
    }

    pub fn page_up(&mut self) {
        self.move_selection(-(self.page_size as i32));
    }

    pub fn page_down(&mut self) {
        self.move_selection(self.page_size as i32);
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.items.get(self.selected_index)
    }
}

pub struct PaginatedLoader<T> {
    pub loaded_items: Vec<T>,
    pub offset: usize,
    pub page_size: usize,
    pub has_more: bool,
    pub loading: bool,
}

impl<T> PaginatedLoader<T> {
    pub fn new(page_size: usize) -> Self {
        Self {
            loaded_items: Vec::new(),
            offset: 0,
            page_size,
            has_more: true,
            loading: false,
        }
    }

    pub fn append(&mut self, mut items: Vec<T>) {
        self.loaded_items.append(&mut items);
        self.offset = self.loaded_items.len();
        self.loading = false;
    }

    pub fn set_has_more(&mut self, has_more: bool) {
        self.has_more = has_more;
    }

    pub fn should_load_more(&self, current_index: usize) -> bool {
        !self.loading && self.has_more && current_index + self.page_size >= self.loaded_items.len()
    }

    pub fn start_loading(&mut self) {
        self.loading = true;
    }

    pub fn reset(&mut self) {
        self.loaded_items.clear();
        self.offset = 0;
        self.has_more = true;
        self.loading = false;
    }
}
