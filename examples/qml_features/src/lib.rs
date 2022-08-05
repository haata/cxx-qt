// SPDX-FileCopyrightText: 2021 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
// SPDX-FileContributor: Gerhard de Clercq <gerhard.declercq@kdab.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

mod data_struct_properties;
mod empty;
mod mock_qt_types;
mod rust_obj_invokables;
mod serialisation;
mod signals;
mod types;

#[cxx_qt::bridge(namespace = "cxx_qt::my_object")]
mod my_object {
    #[namespace = ""]
    unsafe extern "C++" {
        include!("cxx-qt-lib/include/qt_types.h");
        type QString = cxx_qt_lib::QString;
    }

    pub struct Data {
        number: i32,
        string: UniquePtr<QString>,
    }

    impl Default for Data {
        fn default() -> Self {
            Self {
                number: 0,
                string: QString::from_str(""),
            }
        }
    }

    #[derive(Default)]
    pub struct RustObj;

    impl cxx_qt::QObject<RustObj> {
        #[invokable]
        pub fn increment_number_self(&self, cpp: &mut CppObj) {
            let value = cpp.number() + 1;
            cpp.set_number(value);
        }

        #[invokable]
        pub fn increment_number(&self, number: i32) -> i32 {
            number + 1
        }

        #[invokable]
        pub fn say_hi(&self, string: &QString, number: i32) {
            println!(
                "Hi from Rust! String is {} and number is {}",
                string, number
            );
        }
    }
}
