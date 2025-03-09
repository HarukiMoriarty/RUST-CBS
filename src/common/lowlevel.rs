use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) struct LowLevelNode {
    pub(crate) position: (usize, usize), // once we can determine a position, we can also determine the h_cost
    pub(crate) f_open_cost: usize,
    pub(crate) f_focal_cost: usize,
    pub(crate) g_cost: usize,
    pub(crate) time_step: usize, // before reach constraint limit, time_step is exactly same as g_cost
}

// Open List Wrapper
#[derive(Debug)]
pub(crate) struct OpenOrderWrapper(pub(crate) Rc<RefCell<LowLevelNode>>);

impl PartialEq for OpenOrderWrapper {
    fn eq(&self, other: &Self) -> bool {
        let self_node = self.0.borrow();
        let other_node = other.0.borrow();

        // If we have same position, we should have same h_cost, thus we have same f_cost
        self_node.position == other_node.position && self_node.g_cost == other_node.g_cost
    }
}

impl Eq for OpenOrderWrapper {}

impl PartialOrd for OpenOrderWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OpenOrderWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_node = self.0.borrow();
        let other_node = other.0.borrow();

        self_node
            .f_open_cost
            .cmp(&other_node.f_open_cost)
            // Higher g cost (time) has higher priority
            .then_with(|| other_node.g_cost.cmp(&self_node.g_cost))
            // Ticky thing: if g cost is the same, then time step must be same;
            // If time step is the same, g cost might be different
            .then_with(|| self_node.position.cmp(&other_node.position))
    }
}

impl OpenOrderWrapper {
    pub(crate) fn f_open_cost(&self) -> usize {
        self.0.borrow().f_open_cost
    }

    pub(crate) fn from_node(node_rc: &Rc<RefCell<LowLevelNode>>) -> Self {
        OpenOrderWrapper(Rc::clone(node_rc))
    }
}

// Focal List Wrapper
#[derive(Debug)]
pub(crate) struct FocalOrderWrapper(pub(crate) Rc<RefCell<LowLevelNode>>);

impl PartialEq for FocalOrderWrapper {
    fn eq(&self, other: &Self) -> bool {
        let self_node = self.0.borrow();
        let other_node = other.0.borrow();

        // If we have same position, we should have same h_cost, thus we have same f_cost
        self_node.position == other_node.position
            && self_node.g_cost == other_node.g_cost
            && self_node.f_focal_cost == other_node.f_focal_cost
    }
}

impl Eq for FocalOrderWrapper {}

impl PartialOrd for FocalOrderWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FocalOrderWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_node = self.0.borrow();
        let other_node = other.0.borrow();

        self_node
            .f_focal_cost
            .cmp(&other_node.f_focal_cost)
            .then_with(|| self_node.f_open_cost.cmp(&other_node.f_open_cost))
            // Higher g cost has higher priority
            .then_with(|| other_node.g_cost.cmp(&self_node.g_cost))
            // Tricky thing: if g cost is the same, then time step must be same;
            // if time step is the same, g cost might be different.
            .then_with(|| self_node.position.cmp(&other_node.position))
    }
}

impl FocalOrderWrapper {
    pub(crate) fn from_node(node_rc: &Rc<RefCell<LowLevelNode>>) -> Self {
        FocalOrderWrapper(Rc::clone(node_rc))
    }
}

pub(crate) fn create_open_focal_node(
    position: (usize, usize),
    f_open_cost: usize,
    f_focal_cost: usize,
    g_cost: usize,
    time_step: usize,
) -> (OpenOrderWrapper, FocalOrderWrapper) {
    let node = LowLevelNode {
        position,
        f_open_cost,
        f_focal_cost,
        g_cost,
        time_step,
    };

    let node_rc = Rc::new(RefCell::new(node));
    let open_wrapper = OpenOrderWrapper(Rc::clone(&node_rc));
    let focal_wrapper = FocalOrderWrapper(Rc::clone(&node_rc));

    (open_wrapper, focal_wrapper)
}

pub(crate) fn create_open_node(
    position: (usize, usize),
    f_open_cost: usize,
    g_cost: usize,
    time_step: usize,
) -> OpenOrderWrapper {
    let node = LowLevelNode {
        position,
        f_open_cost,
        f_focal_cost: 0, // Default value since not used
        g_cost,
        time_step,
    };

    let node_rc = Rc::new(RefCell::new(node));
    OpenOrderWrapper(Rc::clone(&node_rc))
}

pub(crate) fn create_focal_node(
    position: (usize, usize),
    f_open_cost: usize,
    f_focal_cost: usize,
    g_cost: usize,
    time_step: usize,
) -> FocalOrderWrapper {
    let node = LowLevelNode {
        position,
        f_open_cost,
        f_focal_cost,
        g_cost,
        time_step,
    };

    let node_rc = Rc::new(RefCell::new(node));
    FocalOrderWrapper(Rc::clone(&node_rc))
}
