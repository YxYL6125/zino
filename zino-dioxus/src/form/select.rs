use super::DataEntry;
use crate::{class::Class, format_class};
use dioxus::prelude::*;
use zino_core::SharedString;

/// A control that provides a menu of data entries.
pub fn DataSelect<T: DataEntry + Clone + PartialEq>(props: DataSelectProps<T>) -> Element {
    let class = format_class!(props, "select");
    let fullwidth_class = Class::check("is-fullwidth", props.fullwidth);
    let default_choice = props.options.first().cloned();
    let entries = props.options.clone();
    rsx! {
        div {
            class: "{class} {fullwidth_class}",
            select {
                name: "{props.name}",
                required: props.required,
                onmounted: move |_event| {
                    if let Some(handler) = props.on_select.as_ref() {
                        if let Some(entry) = default_choice.as_ref() {
                            handler.call(entry.clone());
                        }
                    }
                },
                onchange: move |event| {
                    if let Some(handler) = props.on_select.as_ref() {
                        let value = event.value();
                        if let Some(entry) = entries.iter().find(|d| d.value() == value) {
                            handler.call(entry.clone());
                        }
                    }
                },
                for entry in props.options {
                    option {
                        key: "{entry.key()}",
                        value: "{entry.value()}",
                        "{entry.label()}"
                    }
                }
            }
        }
    }
}

/// The [`DataSelect`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct DataSelectProps<T: Clone + PartialEq + 'static> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The data options.
    pub options: Vec<T>,
    /// The name of the control.
    #[props(into)]
    pub name: SharedString,
    /// A flag to determine whether the control is fullwidth or not.
    #[props(default = false)]
    pub fullwidth: bool,
    /// A flag to determine whether the control is required or not.
    #[props(default = false)]
    pub required: bool,
    /// An event handler to be called when the choice is selected.
    pub on_select: Option<EventHandler<T>>,
}
