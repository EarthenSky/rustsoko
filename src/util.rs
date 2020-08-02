use tui::widgets::ListState;

pub struct SimpleMatrix<T> {
    pub width: usize,
    pub data: Vec<T>,
} 

impl<T> SimpleMatrix<T> {
    pub fn new(width: usize, data: Vec<T>) -> SimpleMatrix<T> {
        SimpleMatrix {
            width,
            data,
        } 
    }
    /*
    pub fn get(&self, x: usize, y: usize) -> T {
       self.data[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: T) {
        self.data[y * self.width + x] = val;
    }
    */
}

/*
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
*/