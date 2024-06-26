use crate::{class::Class, format_class, icon::SvgIcon};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaArrowLeft, FaArrowRight};
use zino_core::SharedString;

/// A vertical menu used in the navigation aside.
pub fn Pagination(props: PaginationProps) -> Element {
    let total = props.total;
    let page_size = props.page_size.max(1);
    let current_page = props.current_page.max(1);
    let page_count = total.div_ceil(page_size);
    if total == 0 || page_count <= 1 {
        return None;
    }

    let class = format_class!(props, "pagination");
    let prev_invisible = Class::check("is-invisible", current_page == 1 || page_count <= 5);
    let next_invisible = Class::check(
        "is-invisible",
        total <= current_page * page_size || page_count <= 5,
    );
    let prev_text = props.prev_text.unwrap_or("Previous".into());
    let next_text = props.next_text.unwrap_or("Next".into());
    rsx! {
        nav {
            class: "{class} is-centered",
            a {
                class: "pagination-previous {prev_invisible}",
                onclick: move |_| {
                    if let Some(handler) = props.on_change.as_ref() {
                        handler.call(current_page - 1);
                    }
                },
                if props.prev.is_some() {
                    { props.prev }
                } else {
                    SvgIcon {
                        shape: FaArrowLeft,
                        width: 16,
                    }
                    span {
                        class: "ml-1",
                        "{prev_text}"
                    }
                }
            }
            ul {
                class: "pagination-list",
                if current_page > 2 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(1);
                                }
                            },
                            "1"
                        }
                    }
                }
                if current_page > 3 && page_count > 5 {
                    li {
                        span {
                            class: "pagination-ellipsis",
                            "…"
                        }
                    }
                }
                if current_page > 4 && page_count < current_page + 1 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page - 3);
                                }
                            },
                            "{current_page - 3}"
                        }
                    }
                }
                if current_page > 3 && page_count < current_page + 2 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page - 2);
                                }
                            },
                            "{current_page - 2}"
                        }
                    }
                }
                if current_page > 1 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page - 1);
                                }
                            },
                            "{current_page - 1}"
                        }
                    }
                }
                li {
                    a {
                        class: "pagination-link is-current",
                        onclick: move |_| {
                            if let Some(handler) = props.on_change.as_ref() {
                                handler.call(current_page);
                            }
                        },
                        "{current_page}"
                    }
                }
                if current_page < page_count {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page + 1);
                                }
                            },
                            "{current_page + 1}"
                        }
                    }
                }
                if current_page < 3 && page_count > current_page + 2 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page + 2);
                                }
                            },
                            "{current_page + 2}"
                        }
                    }
                }
                if current_page < 2 && page_count > current_page + 3 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page + 3);
                                }
                            },
                            "{current_page + 3}"
                        }
                    }
                }
                if page_count > current_page + 2 && page_count > 5 {
                    li {
                        span {
                            class: "pagination-ellipsis",
                            "…"
                        }
                    }
                }
                if page_count > current_page + 1 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(page_count);
                                }
                            },
                            "{page_count}"
                        }
                    }
                }
            }
            a {
                class: "pagination-next {next_invisible}",
                onclick: move |_| {
                    if let Some(handler) = props.on_change.as_ref() {
                        handler.call(current_page + 1);
                    }
                },
                if props.next.is_some() {
                    { props.next }
                } else {
                    span {
                        class: "mr-1",
                        "{next_text}"
                    }
                    SvgIcon {
                        shape: FaArrowRight,
                        width: 16,
                    }
                }
            }
        }
    }
}

/// The [`Pagination`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct PaginationProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// Total number of data items.
    pub total: usize,
    /// Number of data items per page.
    #[props(default = 10)]
    pub page_size: usize,
    /// The current page number.
    pub current_page: usize,
    /// The element for the previous button.
    #[props(into, default = "Previous")]
    pub prev_text: Option<SharedString>,
    /// The text for the next button.
    #[props(into, default = "Next")]
    pub next_text: Option<SharedString>,
    /// The element for the previous button.
    pub prev: Option<VNode>,
    /// The element for the next button.
    pub next: Option<VNode>,
    /// An event handler to be called when the page number is changed.
    pub on_change: Option<EventHandler<usize>>,
}
