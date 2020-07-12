#![allow(unused_imports)]
#![no_std]

/// Project a raw pointer's field.
macro_rules! project {
    ($obj:ident.{$($field:ident),*}) => {
        $(project!($obj.$field));+
    };
    ($obj:ident.{$($($field:ident).+ as $name:ident),*}) => {
        $(project!($obj.$($field).+ as $name));+
    };
    ($obj:ident.$($field:ident).+ as $name:ident) => {
        let $name = unsafe { &mut (*$obj).$($field).+ };
    };
    ($obj:ident.$field:ident) => {
        let $field = unsafe { &mut (*$obj).$field };
    }
}

pub mod framebuffer;
pub mod vec2;
pub mod controller;
pub mod graphics;
pub mod foreground;
pub mod background;
pub mod terrain;
pub mod state;
pub mod entity;
pub mod lz4;
