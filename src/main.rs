extern crate egg_mode;
extern crate getopts;
extern crate gdk_pixbuf;
extern crate gdk_pixbuf_sys;
extern crate glib;
extern crate gtk;
extern crate gtk_sys;
extern crate regex;
extern crate rustc_serialize;
extern crate toml;

#[macro_use]
extern crate log;
extern crate env_logger;

use getopts::Options;
use gtk::{Box, Image, Label, ListBox, Paned, Revealer};
use gtk::{ScrolledWindow, Window, WindowType};
use gtk::{ToolButton, ToolItem};
use gtk::Orientation;
use gtk::Switch;

// import gtk
use gtk::prelude::*;

use std::{env, fs};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time;

// module import
mod auth;
mod vars;
mod config;
mod timeline;
mod utils;
mod cache;

pub fn main() {
    // parse flags
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Show this usage message.");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => panic!(err.to_string()),
    };

    if matches.opt_present("h") {
        // usage(opts);
        return;
    }

    // logger initialization
    match env_logger::init() {
        Ok(_) => (),
        Err(err) => {
            error!("{:?}", err);
            panic!("{:?}", err);
        },
    }

    // thread channel setup
    let (tx, rx) = channel();

    // config file initialization
    let home_dir = match env::home_dir() {
        Some(home_dir) => home_dir,
        None => {
            error!("home directory is not set");
            panic!("home directory is not set")
        },
    };

    let cache_dir = home_dir.clone().join(vars::CACHE_DIR).join("rustytwit");
    let cache_image_dir = cache_dir.clone().join("image");
    let config_dir = home_dir.clone().join(vars::CONFIG_DIR).join("rustytwit");
    {
        fs::create_dir_all(cache_dir.clone()).ok();
        fs::create_dir_all(cache_image_dir.clone()).ok();
        fs::create_dir_all(config_dir.clone()).ok();
    }
    let filename = config_dir.clone().join(vars::CONFIG);

    let mut config = match config::Config::new(&filename) {
        Ok(config) => Arc::new(config),
        Err(err) => {
            error!("{:?}", err);
            panic!("{:?}", err)
        },
    };

    // load timeline cache
    let cache_home_timeline = cache_dir.clone().join("home_timeline.json");
    let home_timeline: Vec<timeline::home::TimelineRow> = match cache::load(cache_home_timeline) {
        Ok(home) => home,
        Err(err) => {
            error!("{:?}", err);
            panic!("{:?}", err)
        },
    };
    let home_timeline = Arc::new(Mutex::new(home_timeline));

    // variable initialization
    let consumer_token: egg_mode::Token = egg_mode::Token::new(::vars::CONSUMER_KEY, ::vars::CONSUMER_KEY_SECRET);
    let consumer_token = Arc::new(consumer_token);

    if config.toml.access_key.key == "" || config.toml.access_key.secret == "" {
        let consumer_token = egg_mode::Token::new(::vars::CONSUMER_KEY, ::vars::CONSUMER_KEY_SECRET);
        let access = match auth::authorize(consumer_token) {
            Ok(access) => access,
            Err(err) => {
                error!("{:?}", err);
                panic!("{:?}", err)
            },
        };
        Arc::make_mut(&mut config).toml.access_key.key = access.key.into_owned();
        Arc::make_mut(&mut config).toml.access_key.secret = access.secret.into_owned();
    }
    let access_token: egg_mode::Token = egg_mode::Token::new(config.toml.access_key.key.clone(),
                                                             config.toml.access_key.secret.clone());

    let access_token = Arc::new(access_token);

    // gui initialization
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("rustytwit");

    // vbox setup the entire window
    let vbox = Box::new(Orientation::Vertical, 0);

    let scrolled_window = ScrolledWindow::new(None, None);
    let toolbar = gtk::Toolbar::new();
    toolbar.set_style(gtk::ToolbarStyle::BothHoriz);
    match toolbar.get_style_context() {
        Some(style_context) => style_context.add_class(gtk_sys::GTK_STYLE_CLASS_PRIMARY_TOOLBAR),
        None => {
            error!("toolbar style_context is None");
            panic!("toolbar style_context is None");
        },
    }

    vbox.pack_start(&toolbar, false, false, 0);
    vbox.pack_start(&scrolled_window, true, true, 0);

    // listbox setup
    let listbox = ListBox::new();
    scrolled_window.add(&listbox);

    // side_listbox setup
    let side_listbox = ListBox::new();

    // let timeline_label = Label::new(Some("<b>Timeline</b>"));
    // let home_label = Label::new(Some("Home"));
    // let mention_label = Label::new(Some("Mention"));
    // let favorite_label = Label::new(Some("Favorite"));
    // let dm_label = Label::new(Some("Direct Messages"));
    // let list_label = Label::new(Some("<b>List</b>"));

    // timeline_label.set_padding(16, 6);
    // home_label.set_padding(16, 6);
    // mention_label.set_padding(16, 6);
    // favorite_label.set_padding(16, 6);
    // dm_label.set_padding(16, 6);
    // list_label.set_padding(16, 6);

    // timeline_label.set_xalign(0.0);
    // list_label.set_xalign(0.0);

    // timeline_label.set_use_markup(true);
    // list_label.set_use_markup(true);

    // let timeline = ListBoxRow::new();
    // let home = gtk::EventBox::new();
    // let mention = ListBoxRow::new();
    // let favorite = ListBoxRow::new();
    // let dm = ListBoxRow::new();
    // let list = ListBoxRow::new();

    // timeline.add(&timeline_label);
    // home.add(&home_label);
    // mention.add(&mention_label);
    // favorite.add(&favorite_label);
    // dm.add(&dm_label);
    // list.add(&list_label);

    // timeline.set_selectable(false);
    // list.set_selectable(false);
    // timeline.set_can_focus(false);
    // list.set_can_focus(false);

    // left_list_box.insert(&timeline, -1);
    // left_list_box.insert(&home, -1);
    // left_list_box.insert(&mention, -1);
    // left_list_box.insert(&favorite, -1);
    // left_list_box.insert(&dm, -1);
    // left_list_box.insert(&list, -1);

    // paned setup
    let paned = Paned::new(Orientation::Horizontal);
    paned.pack1(&side_listbox, false, true);
    paned.add2(&vbox);

    // toolbar setup
    let refresh_button_icon = Image::new_from_icon_name("gtk-refresh", 0);
    let refresh_button_label = "refresh";
    let refresh_button = ToolButton::new(Some(&refresh_button_icon), Some(refresh_button_label));
    toolbar.insert(&refresh_button, 0);

    {
        let separator_spacer = gtk::SeparatorToolItem::new();
        separator_spacer.set_expand(true);
        separator_spacer.set_draw(false);
        toolbar.insert(&separator_spacer, 1);
    }

    // definition url filter
    let toolitem_url = ToolItem::new();
    {
        let listbox = listbox.clone();
        let config = config.clone();
        let home = home_timeline.clone();

        let vbox = Box::new(Orientation::Vertical, 0);

        let switch_url = Switch::new();
        switch_url.connect_state_set(move |switch, flag| {
            let mut guard = match home.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            debug!("switch_unread is {}", switch.get_active());
            let timeline = guard.deref_mut();
            timeline::home::fixup_home(timeline, config.toml.home_timeline.limits.get());
            match timeline::home::update_home_timeline(&listbox, timeline, false, flag) {
                Ok(_) => (),
                Err(err) => { error!("{:?}", err); panic!("{:?}", err) },
            }
            debug!("{}", flag);
            return Inhibit(false);
        });
        vbox.pack_start(&switch_url, true, true, 0);

        // let label_text = Label::new(Some("<b>url filter</b>"));
        let label_text = Label::new(Some(""));
        label_text.set_use_markup(true);
        vbox.pack_start(&label_text, true, true, 1);
        toolitem_url.add(&vbox);
    }
    toolbar.insert(&toolitem_url, 2);

    {
        let separator_spacer = gtk::SeparatorToolItem::new();
        separator_spacer.set_expand(false);
        separator_spacer.set_draw(false);
        toolbar.insert(&separator_spacer, 3);
    }

    // definition unread filter
    let toolitem_unread = ToolItem::new();
    {
        let listbox = listbox.clone();
        let config = config.clone();
        let home = home_timeline.clone();

        let vbox = Box::new(Orientation::Vertical, 0);

        let switch_unread = Switch::new();
        switch_unread.connect_state_set(move |switch, flag| {
            let mut guard = match home.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            debug!("switch_unread is {}", switch.get_active());
            let timeline = guard.deref_mut();
            timeline::home::fixup_home(timeline, config.toml.home_timeline.limits.get());
            match timeline::home::update_home_timeline(&listbox, timeline, false, flag) {
                Ok(_) => (),
                Err(err) => {
                    error!("{:?}", err);
                    panic!("{:?}", err)
                },
            }
            return Inhibit(false);
        });
        vbox.pack_start(&switch_unread, true, true, 0);

        let label_text = Label::new(Some("<b>unread filter</b>"));
        label_text.set_use_markup(true);
        vbox.pack_start(&label_text, true, true, 1);
        toolitem_unread.add(&vbox);
    }
    toolbar.insert(&toolitem_unread, 4);

    let separator_bar = gtk::SeparatorToolItem::new();
    separator_bar.set_draw(true);
    toolbar.insert(&separator_bar, 5);

    let pref_button_icon = Image::new_from_icon_name("gtk-preferences", 1);
    let pref_button_label = "preferences";
    let pref_button = ToolButton::new(Some(&pref_button_icon), Some(pref_button_label));
    toolbar.insert(&pref_button, 6);

    // window setup
    window.add(&paned);
    window.show_all();
    {
        let config = config.clone();
        let listbox = listbox.clone();

        let mut guard = match home_timeline.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut timeline = guard.deref_mut();
        timeline::home::fixup_home(timeline, config.toml.home_timeline.limits.get());
        match timeline::home::update_home_timeline(&listbox, timeline, false, false) {
            Ok(_) => (),
            Err(err) => {
                error!("{:?}", err);
                panic!("{:?}", err)
            },
        }
    }

    // event definition, when refresh_button is clicked
    {
        let listbox = listbox.clone();

        let config = config.clone();

        let consumer_token = consumer_token.clone();
        let access_token = access_token.clone();

        let home = home_timeline.clone();

        refresh_button.connect_clicked(move |_| {
            let ref consumer_token = *consumer_token.as_ref();
            let ref access_token = *access_token.as_ref();
            // let mut param = ParamList::new();
            // param.insert(Cow::Owned("count".to_string()),
            //              Cow::Owned(format!("{}", 200)));
            // param.insert(Cow::Owned("since_id".to_string()),
            //              Cow::Owned(format!("{}", config.toml.home_timeline.last_update_id.get())));
            let tweets = match timeline::home::get_home_timeline(consumer_token, access_token) {
                Ok(tweets) => tweets,
                Err(_) => return,
            };
            match tweets.first() {
                Some(row) => config.toml.home_timeline.last_update_id.set(row.tweet.id),
                None => (),
            };

            // add tweets to home_timeline and update home_timeline
            {
                let mut guard = match home.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                let mut index = 0;
                for row in tweets.clone() {
                    guard.insert(index, row.clone());
                    index += 1;
                }
                let mut timeline = guard.deref_mut();
                timeline::home::fixup_home(timeline, config.toml.home_timeline.limits.get());
            }
            match timeline::home::update_home_timeline(&listbox, &tweets, true, false) {
                Ok(_) => (),
                Err(err) => {
                    error!("{:?}", err);
                    panic!("{:?}", err)
                },
            }
        });
    }

    // event definition, when listboxrow is selected
    {
        let home_timeline = home_timeline.clone();
        let config = config.clone();

        listbox.connect_row_selected(move |_, listboxrow| {
            let revealer = match listboxrow.clone() {
                Some(listboxrow) => {
                    match listboxrow.get_child() {
                        Some(widget) => {
                            match widget.downcast::<Revealer>() {
                                Ok(revealer) => revealer,
                                Err(err) => {
                                    error!("{:?}", err);
                                    return;
                                },
                            }
                        },
                        None => return,
                    }
                },
                None => return,
            };
            let box_revealer = match revealer.get_child() {
                Some(widget) => {
                    match widget.downcast::<Box>() {
                        Ok(box_revealer) => box_revealer,
                        Err(err) => {
                            error!("{:?}", err);
                            return;
                        },
                    }
                },
                None => return,
            };
            let unread_image = match box_revealer.get_children()[2].clone().downcast::<Image>() {
                Ok(unread_image) => unread_image,
                Err(err) => {
                    error!("{:?}", err);
                    return;
                },
            };
            let id_label = match box_revealer.get_children()[4].clone().downcast::<Label>() {
                Ok(id_label) => id_label,
                Err(err) => {
                    error!("{:?}", err);
                    return;
                },
            };

            // clear unread image to empty
            unread_image.clear();
            unread_image.set_padding(vars::UNREAD_IMAGE_SIZE, vars::UNREAD_IMAGE_SIZE);

            // update config.toml.home_timeline.last_read_id
            let id: i64 = match id_label.get_text() {
                Some(id_str) => {
                    match i64::from_str_radix(&id_str, 10) {
                        Ok(id) => id,
                        Err(err) => {
                            error!("{}", err);
                            return;
                        },
                    }
                },
                None => return,
            };
            // update home_timeline unread flag to clear
            {
                let listboxrow = match listboxrow.clone() {
                    Some(listboxrow) => listboxrow,
                    None => return,
                };

                let mut guard = match home_timeline.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                let timeline = guard.deref_mut();

                for row in timeline {
                    if row.tweet.id == id {
                        row.unread = false;
                        // expand selected listboxrow
                        listboxrow.remove(&revealer);
                        let revealer = match timeline::home::create_expanded_revealer(row.clone()) {
                            Ok(revealer) => revealer,
                            Err(err) => {
                                error!("{:?}", err);
                                return;
                            }, 
                        };
                        listboxrow.add(&revealer);
                        match timeline::home::show_listboxrow(&listboxrow) {
                            Ok(_) => (),
                            Err(err) => {
                                error!("{:?}", err);
                                return;
                            }, 
                        }
                        break;
                    }
                }
                config.toml.home_timeline.last_read_id.set(id);
            }
        });
    }

    {
        // create threads send signal, update timeline
        let config = config.clone();

        let consumer_token = consumer_token.clone();
        let access_token = access_token.clone();

        thread::spawn(move || {
            let ref consumer_token = consumer_token.as_ref();
            let ref access_token = access_token.as_ref();

            loop {
                let retry_secs = 60;
                let duration = 600;
                // let mut param = ParamList::new();
                // param.insert(Cow::Owned("count".to_string()),
                //              Cow::Owned(format!("{}", config.toml.home_timeline.limits.get())));
                // param.insert(Cow::Owned("since_id".to_string()),
                //              Cow::Owned(format!("{}", config.toml.home_timeline.last_update_id.get())));

                let timeline = match timeline::home::get_home_timeline(consumer_token, access_token) {
                    Ok(timeline) => timeline,
                    Err(err) => {
                        error!("{:?}", err);
                        error!("try later, after {} seconds", retry_secs);
                        thread::sleep(time::Duration::from_secs(retry_secs));
                        continue;
                    },
                };
                info!("thread: get_home_timeline()");
                match timeline.first() {
                    Some(row) => {
                        let id = row.tweet.id;
                        config.toml.home_timeline.last_update_id.set(id);
                        match tx.send(timeline.clone()) {
                            Ok(_) => (),
                            Err(err) => {
                                error!("{:?}", err);
                                panic!("{:?}", err);
                            },
                        }
                    },
                    None => {
                        debug!("thread: timeline is None");
                    },
                };
                thread::sleep(time::Duration::from_secs(duration));
            }
        });
    }
    {
        // receive signal from threads, update gtk widgets thread
        let config = config.clone();
        let home = home_timeline.clone();

        let listbox = listbox.clone();

        let refresh_timeline = move || {
            debug!("pooling try to receive from channel");
            let tweets = match rx.try_recv() {
                Ok(tweets) => tweets,
                Err(_) => return glib::Continue(true),
            };
            {
                let mut guard = match home.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                let mut index = 0;
                for row in tweets.clone() {
                    guard.insert(index, row.clone());
                    index += 1;
                }
                let mut timeline = guard.deref_mut();
                timeline::home::fixup_home(timeline, config.toml.home_timeline.limits.get());
            }
            let _ = timeline::home::update_home_timeline(&listbox, &tweets, true, false);

            return glib::Continue(true);
        };

        gtk::timeout_add_seconds(10, refresh_timeline);
    }

    // exit progress, when program exit
    {
        let config = config.clone();

        window.connect_delete_event(move |_, _| {
            gtk::main_quit();

            // synchronize config to config file
            match config.sync() {
                Ok(_) => (),
                Err(err) => {
                    error!("{:?}", err);
                    panic!("{:?}", err)
                },
            }

            // write cache to each cache file
            {
                let cache_home = cache_dir.clone().join(vars::CACHE_HOME);
                let guard = match home_timeline.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                match cache::write(cache_home, &guard) {
                    Ok(_) => (),
                    Err(err) => {
                        error!("{:?}", err);
                        panic!("{:?}", err)
                    },
                }
            }

            Inhibit(false)
        });
    }
    gtk::main();
}
