use gtk::{
    glib::{Cast, IsA, ObjectType, StaticType},
    prelude::WidgetExt,
};

pub trait WidgetUtilsExt {
    /// Disposes all children of a widget
    fn dispose_children(&self);

    fn parent_of_type<P: StaticType + ObjectType + IsA<gtk::Widget>>(&self) -> Option<P>;
}

impl<T> WidgetUtilsExt for T
where
    T: WidgetExt,
{
    fn parent_of_type<P: StaticType + ObjectType + IsA<gtk::Widget>>(&self) -> Option<P> {
        let mut iterator: gtk::Widget = self.clone().upcast();

        while let Some(parent) = iterator.parent() {
            if let Some(parent) = parent.downcast_ref::<P>() {
                return Some(parent.clone());
            }

            iterator = parent;
        }

        None
    }

    fn dispose_children(&self) {
        while let Some(child) = self.first_child() {
            child.unparent();
        }
    }
}
