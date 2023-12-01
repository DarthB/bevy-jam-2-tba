use bevy::prelude::*;

/// Enumeration that differentiates between the kinds of field elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum FieldElementKind {
    /// the area is empty
    #[default]
    Empty,

    /// the area is outside the movable region but rendering may occur here
    OutOfMovableRegion,

    /// the area is complety outside the valid region,
    OutOfValidRegion,

    /// the area is blocked by a block and the parent blob of the block entity may be given here
    Block(Option<Entity>),

    /// the area is occupied by a tool, if that is true it may be at the same time occupied by a block
    Tool(Entity),
}

/// a element that descirbes a coordinate in the FieldState
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub struct FieldElement {
    /// flag indicating if that field element is a target for the level
    pub is_target: bool,

    /// "most" relating entity, if a tool and a blob occupy the space this contains the tool
    pub entity: Option<Entity>,

    /// enumeration to distinguish between types of field elements
    pub kind: FieldElementKind,

    /// the relative position of the element in respect to the fields coordinate system
    pub position: IVec2,
}

/// Encapsules the game state of a game field.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Reflect)]
pub struct FieldState {
    // gives the start values and end values of coordinates in this field
    bounds: (IVec2, IVec2),
    /// Field elements.
    elements: Vec<FieldElement>,
}

impl FieldState {
    pub fn new(bounds: (IVec2, IVec2)) -> Self {
        if bounds.0.x > 0 || bounds.0.x >= bounds.1.x || bounds.0.y > 0 || bounds.0.y >= bounds.1.y
        {
            panic!("Illegal bounds given for FactoryFieldState")
        }

        let len = len_from_bounds(bounds);
        let mut reval = FieldState {
            bounds,
            elements: vec![FieldElement::default(); len.0 * len.1],
        };

        // initialize the position values
        for x in bounds.0.x..bounds.1.x {
            for y in bounds.0.y..bounds.1.y {
                let pos = IVec2::new(x, y);
                let mut el = reval.get_element(pos).unwrap();
                el.position = pos;
                reval.set_element(pos, el);
            }
        }

        reval
    }

    pub fn len(&self) -> (usize, usize) {
        len_from_bounds(self.bounds)
    }

    pub fn bounds(&self) -> (IVec2, IVec2) {
        self.bounds
    }

    pub fn get_element(&self, coord: IVec2) -> Option<FieldElement> {
        self.coord_to_idx(coord).map(|idx| self.elements[idx])
    }

    pub fn set_element(&mut self, coord: IVec2, new_el: FieldElement) -> bool {
        if let Some(idx) = self.coord_to_idx(coord) {
            self.elements[idx] = new_el;
            true
        } else {
            false
        }
    }

    pub fn get_occupied_coordinates(&self) -> Vec<IVec2> {
        let mut reval = vec![];
        for el in self.into_iter() {
            match el.kind {
                FieldElementKind::OutOfMovableRegion | FieldElementKind::Block(_) => {
                    reval.push(el.position);
                }
                _ => {}
            }
        }
        reval
    }

    pub fn are_all_coordinates(
        &self,
        coords: &Vec<IVec2>,
        exceptions: Option<&Vec<IVec2>>,
        predicate: &dyn Fn(&FieldElement) -> bool,
    ) -> bool {
        let exceptions = exceptions.map(|e| (e, true));
        for v in coords {
            let res = self.predicate_at_coordinate(*v, exceptions, predicate);
            if !res {
                return false;
            }
        }
        true
    }

    pub fn is_any_coordinate(
        &self,
        coords: &Vec<IVec2>,
        exceptions: Option<&Vec<IVec2>>,
        predicate: &dyn Fn(&FieldElement) -> bool,
    ) -> bool {
        let exceptions = exceptions.map(|e| (e, false));

        for v in coords {
            let res = self.predicate_at_coordinate(*v, exceptions, predicate);
            if res {
                return true;
            }
        }
        false
    }

    pub fn predicate_at_coordinate(
        &self,
        coord: IVec2,
        exceptions: Option<(&Vec<IVec2>, bool)>,
        predicate: &dyn Fn(&FieldElement) -> bool,
    ) -> bool {
        if let Some(exceptions) = exceptions {
            if exceptions.0.contains(&coord) {
                return exceptions.1;
            }
        }
        if let Some(element) = self.get_element(coord) {
            predicate(&element)
        } else {
            let el = FieldElement {
                entity: None,
                kind: FieldElementKind::OutOfValidRegion,
                position: coord,
                is_target: false,
            };
            predicate(&el)
        }
    }

    fn coord_to_idx(&self, coord: IVec2) -> Option<usize> {
        if coord.x < self.bounds.0.x
            || coord.x >= self.bounds.1.x
            || coord.y < self.bounds.0.y
            || coord.y >= self.bounds.1.y
        {
            None
        } else {
            let len = self.len();
            let x = (coord.x + self.bounds.0.x.abs()) as usize;
            let y = (coord.y + self.bounds.0.y.abs()) as usize;

            Some(x + y * len.0)
        }
    }
}

fn len_from_bounds(bounds: (IVec2, IVec2)) -> (usize, usize) {
    let dimensions = (bounds.0 - bounds.1).abs();
    (dimensions.x as usize, dimensions.y as usize)
}

impl<'a> IntoIterator for &'a FieldState {
    type Item = FieldElement;

    type IntoIter = FieldElementIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FieldElementIterator {
            field_state: self,
            index: 0,
        }
    }
}

pub struct FieldElementIterator<'a> {
    field_state: &'a FieldState,
    index: usize,
}

impl<'a> Iterator for FieldElementIterator<'a> {
    type Item = FieldElement;

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.index < self.field_state.elements.len() {
            Some(self.field_state.elements[self.index])
        } else {
            None
        };
        self.index += 1;
        res
    }
}
