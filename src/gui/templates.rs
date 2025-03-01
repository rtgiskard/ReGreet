// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Templates for various GUI components
#![allow(dead_code)] // Silence dead code warnings for UI code that isn't dead

use gtk::prelude::*;
use relm4::{gtk, WidgetTemplate};

/// Button that ends the greeter (eg. Reboot)
#[relm4::widget_template(pub)]
impl WidgetTemplate for EndButton {
    view! {
        gtk::Button {
            set_focusable: true,
            add_css_class: "destructive-action",
        }
    }
}

/// Label for an entry/combo box
#[relm4::widget_template(pub)]
impl WidgetTemplate for EntryLabel {
    view! {
        gtk::Label {
            set_ellipsize: gtk::pango::EllipsizeMode::Middle,
            set_max_width_chars: 12,
            set_width_request: 100,
            set_xalign: 1.0,
        }
    }
}

/// Main UI of the greeter
#[relm4::widget_template(pub)]
impl WidgetTemplate for Ui {
    view! {
        gtk::Overlay {
            /// Background image
            #[name = "background"]
            gtk::Picture,

            /// Main login box
            add_overlay = &gtk::Box {
                add_css_class: "center",
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                gtk::Grid {
                    set_column_spacing: 15,
                    set_margin_bottom: 15,
                    set_margin_end: 15,
                    set_margin_start: 15,
                    set_margin_top: 15,
                    set_row_spacing: 15,
                    set_width_request: 500,

                    /// Widget to display messages to the user
                    #[name = "message_label"]
                    attach[0, 0, 3, 1] = &gtk::Label {
                        add_css_class: "message",
                        set_margin_bottom: 15,

                        // Format all messages in boldface.
                        #[wrap(Some)]
                        set_attributes = &gtk::pango::AttrList {
                            insert: {
                                let mut font_desc = gtk::pango::FontDescription::new();
                                font_desc.set_weight(gtk::pango::Weight::Bold);
                                gtk::pango::AttrFontDesc::new(&font_desc)
                            },
                        },
                    },

                    #[template]
                    attach[0, 1, 1, 1] = &EntryLabel { set_label: "User:" },

                    /// Label for the sessions widget
                    #[name = "session_label"]
                    #[template]
                    attach[0, 2, 1, 1] = &EntryLabel { set_label: "Session:" },

                    /// Widget containing the usernames
                    #[name = "usernames_box"]
                    attach[1, 1, 1, 1] = &gtk::ComboBoxText { set_hexpand: true },

                    /// Widget where the user enters the username
                    #[name = "username_entry"]
                    attach[1, 1, 1, 1] = &gtk::Entry { set_hexpand: true },

                    /// Widget containing the sessions
                    #[name = "sessions_box"]
                    attach[1, 2, 1, 1] = &gtk::ComboBoxText,

                    /// Widget where the user enters the session
                    #[name = "session_entry"]
                    attach[1, 2, 1, 1] = &gtk::Entry,

                    /// Label for the password widget
                    #[name = "input_label"]
                    #[template]
                    attach[0, 2, 1, 1] = &EntryLabel,

                    /// Widget where the user enters a secret
                    #[name = "secret_entry"]
                    attach[1, 2, 1, 1] = &gtk::PasswordEntry { set_show_peek_icon: true },

                    /// Widget where the user enters something visible
                    #[name = "visible_entry"]
                    attach[1, 2, 1, 1] = &gtk::Entry,

                    /// Button to toggle manual user entry
                    #[name = "user_toggle"]
                    attach[2, 1, 1, 1] = &gtk::ToggleButton {
                        set_icon_name: "document-edit-symbolic",
                        set_tooltip_text: Some("Manually enter username"),
                    },

                    /// Button to toggle manual session entry
                    #[name = "sess_toggle"]
                    attach[2, 2, 1, 1] = &gtk::ToggleButton {
                        set_icon_name: "document-edit-symbolic",
                        set_tooltip_text: Some("Manually enter session command"),
                    },

                    /// Collection of action buttons (eg. Login)
                    attach[1, 3, 2, 1] = &gtk::Box {
                        set_halign: gtk::Align::End,
                        set_spacing: 15,

                        /// Button to cancel password entry
                        #[name = "cancel_button"]
                        gtk::Button {
                            set_focusable: true,
                            set_label: "Cancel",
                            add_css_class: "cancel",
                        },

                        /// Button to enter the password and login
                        #[name = "login_button"]
                        gtk::Button {
                            set_focusable: true,
                            set_label: "Login",
                            set_receives_default: true,
                            add_css_class: "suggested-action",
                            add_css_class: "login",
                        },
                    },
                },
            },

            /// Clock widget
            #[name = "clock_box"]
            add_overlay = &gtk::Box {
                add_css_class: "top",
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Start,
            },

            /// Collection of widgets appearing at the bottom
            add_overlay = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
                set_margin_bottom: 15,
                set_spacing: 15,

                /// Notification bar for error messages
                #[name = "error_info"]
                gtk::InfoBar {
                    add_css_class: "error",

                    // During init, the info bar closing animation is shown. To hide that, make
                    // it invisible. Later, the code will permanently make it visible, so that
                    // `InfoBar::set_revealed` will work properly with animations.
                    set_visible: false,
                    set_message_type: gtk::MessageType::Error,

                    /// The actual error message
                    #[name = "error_label"]
                    gtk::Label {
                        set_halign: gtk::Align::Center,
                        set_margin_top: 10,
                        set_margin_bottom: 10,
                        set_margin_start: 10,
                        set_margin_end: 10,
                    },
                },

                /// Collection of buttons that close the greeter (eg. Reboot)
                gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_homogeneous: true,
                    set_spacing: 15,

                    /// Button to reboot
                    #[name = "reboot_button"]
                    #[template]
                    EndButton { set_label: "Reboot" },

                    /// Button to power-off
                    #[name = "poweroff_button"]
                    #[template]
                    EndButton { set_label: "Power Off" },
                },
            },
        }
    }
}
