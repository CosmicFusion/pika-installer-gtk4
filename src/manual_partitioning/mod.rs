// Use libraries
use std::collections::HashMap;
use std::thread;
/// Use all gtk4 libraries (gtk4 -> gtk because cargo)
/// Use all libadwaita libraries (libadwaita -> adw because cargo)
use gtk::prelude::*;
use gtk::*;
use adw::prelude::*;
use adw::*;
use glib::*;
use gdk::Display;
use gtk::subclass::{layout_child, window};

use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use duct::*;

use std::{
    hash::{
        Hash,
    },
    collections::{
        HashSet
    },
    io::{
        BufRead,
        BufReader,
    },
    process::{
        Command,
        Stdio,
    },
    time::{
        Instant,
        Duration,
    },
    fs,
    path::{
        Path,
    },
};
use std::ops::{Deref, DerefMut};
use duct::cmd;
use gtk::Orientation::Vertical;

use pretty_bytes::converter::convert;
use crate::drive_mount_row::DriveMountRow;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct DriveMount {
    partition: String,
    mountpoint: String,
    mountopt: String,
}

fn create_mount_row(listbox: &gtk::ListBox, manual_drive_mount_array: &Rc<RefCell<Vec<DriveMount>>>, check_part_unique: &Rc<RefCell<bool>>) -> DriveMountRow {
    let partition_scroll_child = gtk::ListBox::builder()
        .build();

    let partitions_scroll = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&partition_scroll_child)
        .build();

    // Create row
    let row = DriveMountRow::new_with_scroll(&partitions_scroll);

    let null_checkbutton = gtk::CheckButton::builder()
        .build();

    let partition_method_manual_emitter = gtk::SignalAction::new("partchg");

    let partition_method_manual_get_partitions_cmd = cmd!("bash", "-c", "sudo /usr/lib/pika/pika-installer-gtk4/scripts/partition-utility.sh get_partitions");
    let partition_method_manual_get_partitions_reader = partition_method_manual_get_partitions_cmd.stderr_to_stdout().reader();
    let mut partition_method_manual_get_partitions_lines = BufReader::new(partition_method_manual_get_partitions_reader.unwrap()).lines();

    for partition in partition_method_manual_get_partitions_lines {
        let partition = partition.unwrap();
        let partition_size_cli = Command::new("sudo")
            .arg("/usr/lib/pika/pika-installer-gtk4/scripts/partition-utility.sh")
            .arg("get_part_size")
            .arg(partition.clone())
            .output()
            .expect("failed to execute process");
        let partition_fs_cli = Command::new("sudo")
            .arg("/usr/lib/pika/pika-installer-gtk4/scripts/partition-utility.sh")
            .arg("get_part_fs")
            .arg(partition.clone().replace("mapper/", ""))
            .output()
            .expect("failed to execute process");
        let partition_size = String::from_utf8(partition_size_cli.stdout).expect("Failed to create float").trim().parse::<f64>().unwrap();
        let partition_button = gtk::CheckButton::builder()
            .valign(Align::Center)
            .can_focus(false)
            .build();
        partition_button.set_group(Some(&null_checkbutton));
        let partition_row = adw::ActionRow::builder()
            .activatable_widget(&partition_button)
            .title(partition.clone())
            .name(partition.clone())
            .subtitle(String::from_utf8(partition_fs_cli.stdout).expect("Failed read stdout") + &pretty_bytes::converter::convert(partition_size))
            .build();
        partition_row.add_prefix(&partition_button);
        partition_button.connect_toggled(clone!(@weak row, @weak listbox, @weak partition_button, @strong manual_drive_mount_array, @strong partition=> move |_| {
            let mut manual_drive_mount_array_ref = RefCell::borrow_mut(&manual_drive_mount_array);
            if partition_button.is_active() == true {
                row.set_partition(partition.clone());
            } else {
                let manual_drive_mount_array_ref_index = manual_drive_mount_array_ref.iter().position(|x| *x.partition == partition.clone()).unwrap();
                manual_drive_mount_array_ref.remove(manual_drive_mount_array_ref_index);
            }
        }));
        partition_scroll_child.append(&partition_row);
    }

    let listbox_clone = listbox.clone();
    row.connect_closure(
        "row-deleted",
        false,
        closure_local!(@strong row => move |row: DriveMountRow| {
                    listbox_clone.remove(&row)
        })
    );

    // Return row
    row
}

fn has_unique_elements<T>(iter: T) -> bool
    where
        T: IntoIterator,
        T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}

//pub fn manual_partitioning(window: &adw::ApplicationWindow, partitioning_stack: &gtk::Stack, bottom_next_button: &gtk::Button) -> (gtk::TextBuffer, gtk::TextBuffer, adw::PasswordEntryRow) {
pub fn manual_partitioning(window: &adw::ApplicationWindow, partitioning_stack: &gtk::Stack, bottom_next_button: &gtk::Button, manual_drive_mount_array: Rc<RefCell<Vec<DriveMount>>>) {

    let check_part_unique = Rc::new(RefCell::new(true));

    let partition_method_manual_main_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(15)
        .margin_top(15)
        .margin_end(15)
        .margin_start(15)
        .build();

    let partition_method_manual_header_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    // the header text for the partitioning page
    let partition_method_manual_header_text = gtk::Label::builder()
        .label("Manual Partitioning Installer")
        .halign(gtk::Align::End)
        .hexpand(true)
        .margin_top(15)
        .margin_bottom(15)
        .margin_start(15)
        .margin_end(5)
        .build();
    partition_method_manual_header_text.add_css_class("header_sized_text");

    // the header icon for the partitioning icon
    let partition_method_manual_header_icon = gtk::Image::builder()
        .icon_name("input-tablet")
        .halign(gtk::Align::Start)
        .hexpand(true)
        .pixel_size(78)
        .margin_top(15)
        .margin_bottom(15)
        .margin_start(0)
        .margin_end(15)
        .build();

    let partition_method_manual_selection_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let partition_method_manual_gparted_button_content_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let partition_method_manual_gparted_button_content_text = gtk::Label::builder()
        .label("Use this utility to partition/mount/format your drives.")
        .build();

    let partition_method_manual_gparted_button_content = adw::ButtonContent::builder()
        .label("Open GPARTED")
        .icon_name("gparted")
        .build();

    let partition_method_manual_gparted_button = gtk::Button::builder()
        .child(&partition_method_manual_gparted_button_content_box)
        .halign(Align::Center)
        .valign(Align::Start)
        .build();

    let drive_mounts_adw_listbox = gtk::ListBox::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    drive_mounts_adw_listbox.add_css_class("boxed-list");

    let drive_mounts_viewport = gtk::ScrolledWindow::builder()
        .halign(Align::Center)
        .valign(Align::Center)
        .margin_top(30)
        .margin_bottom(30)
        .margin_start(30)
        .margin_end(30)
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .min_content_height(200)
        .min_content_width(200)
        .hexpand(true)
        .vexpand(true)
        .child(&drive_mounts_adw_listbox)
        .build();

    let drive_mount_add_button = gtk::Button::builder()
        .icon_name("list-add")
        .vexpand(true)
        .hexpand(true)
        .build();

    let partition_method_manual_error_label = gtk::Label::builder()
        .halign(Align::Start)
        .valign(Align::End)
        .vexpand(true)
        .visible(false)
        .build();
    partition_method_manual_error_label.add_css_class("small_error_text");

    let partition_method_manual_warn_label = gtk::Label::builder()
        .halign(Align::Start)
        .valign(Align::End)
        .vexpand(true)
        .visible(false)
        .build();
    partition_method_manual_warn_label.add_css_class("small_warn_text");

    partition_method_manual_header_box.append(&partition_method_manual_header_text);
    partition_method_manual_header_box.append(&partition_method_manual_header_icon);
    partition_method_manual_main_box.append(&partition_method_manual_header_box);
    partition_method_manual_main_box.append(&partition_method_manual_selection_box);
    partition_method_manual_gparted_button_content_box.append(&partition_method_manual_gparted_button_content);
    partition_method_manual_gparted_button_content_box.append(&partition_method_manual_gparted_button_content_text);
    partition_method_manual_main_box.append(&partition_method_manual_gparted_button);
    drive_mounts_adw_listbox.append(&drive_mount_add_button);
    partition_method_manual_main_box.append(&drive_mounts_viewport);
    partition_method_manual_main_box.append(&partition_method_manual_error_label);
    partition_method_manual_main_box.append(&partition_method_manual_warn_label);

    partition_method_manual_gparted_button.connect_clicked(move |_| {
        Command::new("gparted")
            .spawn()
            .expect("gparted failed to start");
    });

    drive_mount_add_button.connect_clicked(clone!(@weak drive_mounts_adw_listbox, @strong manual_drive_mount_array, @strong  check_part_unique => move |_| {
        drive_mounts_adw_listbox.append(&create_mount_row(&drive_mounts_adw_listbox, &manual_drive_mount_array, &check_part_unique))
    }));

    let (anti_dup_partition_sender, anti_dup_partition_receiver) = async_channel::unbounded();
    let anti_dup_partition_sender = anti_dup_partition_sender.clone();
    // The long running operation runs now in a separate thread
    gio::spawn_blocking(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            anti_dup_partition_sender
                .send_blocking(true)
                .expect("The channel needs to be open.");
        }
    });

    let anti_dup_partition_loop_context = MainContext::default();
    anti_dup_partition_loop_context.spawn_local(clone!(@weak drive_mounts_adw_listbox, @strong manual_drive_mount_array, @strong  check_part_unique => async move {
        while let Ok(_state) = anti_dup_partition_receiver.recv().await {
            let mut counter = drive_mounts_adw_listbox.first_child();

            let mut manual_drive_mount_array_ref = manual_drive_mount_array.borrow_mut();

            // usage of while loop
            manual_drive_mount_array_ref.clear();
            while let Some(row) = counter {
                if row.widget_name() == "DriveMountRow" {
                    let row_mount = DriveMount {
                        partition: row.clone().property("partition"),
                        mountpoint: row.clone().property("mountpoint"),
                        mountopt: row.clone().property("mountopt"),
                    };
                    manual_drive_mount_array_ref.push(row_mount);
                }
                counter = row.next_sibling();
            }

            let mut counter = drive_mounts_adw_listbox.first_child();
            while let Some(ref row) = counter {
                if row.widget_name() == "DriveMountRow" {
                    let mut counter_scrw = row.property::<gtk::ScrolledWindow>("partitionscroll").child().unwrap().first_child().unwrap().first_child();
                    while let Some(ref row_scrw) = counter_scrw {
                        if manual_drive_mount_array_ref.iter().any(|e| {
                            if !e.partition.is_empty() {
                                row_scrw.widget_name().contains(&e.partition)
                            } else {
                                return false
                            }
                        }) {

                            if *check_part_unique.borrow_mut() == true {
                                row_scrw.set_sensitive(false)
                            } else {
                                row_scrw.set_sensitive(true)
                            }
                        } else {
                            row_scrw.set_sensitive(true)
                        }
                        counter_scrw = row_scrw.next_sibling();
                    }
                }
                counter = row.next_sibling();
            }
        }
    }));

    partitioning_stack.add_titled(&partition_method_manual_main_box, Some("partition_method_manual_page"), "partition_method_manual_page");

    //return(partition_method_manual_target_buffer, partition_method_manual_luks_buffer, partition_method_manual_luks_password_entry)
}


fn partition_err_check(partition_method_manual_warn_label: &gtk::Label,partition_method_manual_error_label: &gtk::Label, manual_drive_mount_array: &Rc<RefCell<Vec<DriveMount>>>, check_part_unique: &Rc<RefCell<bool>>) {
    let mut manual_drive_mount_array_ref = manual_drive_mount_array.borrow_mut();
    if manual_drive_mount_array_ref.len() - manual_drive_mount_array_ref.iter().map(|x| x.mountpoint.as_str()).collect::<HashSet<&str>>().len() > 0 {
        partition_method_manual_error_label.set_label("Multiple drives were mounted to the same mountpoint.");
        partition_method_manual_error_label.set_visible(true);
    } else {
        if partition_method_manual_error_label.label() == "Multiple drives were mounted to the same mountpoint." {
            partition_method_manual_error_label.set_visible(false);
        }
    }

    *check_part_unique.borrow_mut()=true;
    for mountopts in manual_drive_mount_array_ref.iter().map(|x| x.mountopt.as_str()).collect::<HashSet<&str>>() {
        if mountopts.contains("subvol") {
            *check_part_unique.borrow_mut()=false
        }
    }

    if *check_part_unique.borrow_mut() == false {
        partition_method_manual_warn_label.set_label("Partition reuse check will be skipped due to subvol usage.");
        partition_method_manual_warn_label.set_visible(true);
    } else {
        partition_method_manual_warn_label.set_visible(false);
    }
}