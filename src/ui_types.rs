use slint::SharedString;
use crate::models::live::Category as Cat;

#[derive(Clone)]
pub struct Category {
    pub category_id: SharedString,
    pub category_name: SharedString,
    pub parent_id: i32,
}

impl From<crate::models::live::Category> for Category {
    fn from(category: crate::models::live::Category) -> Self {
        Self {
            category_id: category.category_id.into(),
            category_name: category.category_name.into(),
            parent_id: category.parent_id as i32,
        }
    }
}