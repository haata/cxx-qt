// SPDX-FileCopyrightText: 2022 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::generator::{
    cpp::{qobject::GeneratedCppQObjectBlocks, signal::generate_cpp_signals},
    naming::{property::QPropertyName, qobject::QObjectName},
    utils::cpp::syn_type_to_cpp_type,
};
use crate::parser::{mappings::ParsedCxxMappings, property::ParsedQProperty};
use syn::Result;

mod getter;
mod meta;
mod setter;
mod signal;

pub fn generate_cpp_properties(
    properties: &Vec<ParsedQProperty>,
    qobject_idents: &QObjectName,
    cxx_mappings: &ParsedCxxMappings,
) -> Result<GeneratedCppQObjectBlocks> {
    let mut generated = GeneratedCppQObjectBlocks::default();
    let mut signals = vec![];
    let qobject_ident = qobject_idents.cpp_class.cpp.to_string();

    for property in properties {
        // Cache the idents as they are used in multiple places
        let idents = QPropertyName::from(property);
        let cxx_ty = syn_type_to_cpp_type(&property.ty, cxx_mappings)?;

        generated.metaobjects.push(meta::generate(&idents, &cxx_ty));
        generated
            .methods
            .push(getter::generate(&idents, &qobject_ident, &cxx_ty));
        generated
            .private_methods
            .push(getter::generate_wrapper(&idents, &cxx_ty));
        generated
            .methods
            .push(setter::generate(&idents, &qobject_ident, &cxx_ty));
        generated
            .private_methods
            .push(setter::generate_wrapper(&idents, &cxx_ty));
        signals.push(signal::generate(&idents, qobject_idents));
    }

    generated.append(&mut generate_cpp_signals(
        &signals,
        qobject_idents,
        cxx_mappings,
    )?);

    Ok(generated)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::generator::naming::qobject::tests::create_qobjectname;
    use crate::CppFragment;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;
    use quote::format_ident;
    use syn::parse_quote;

    #[test]
    fn test_generate_cpp_properties() {
        let properties = vec![
            ParsedQProperty {
                ident: format_ident!("trivial_property"),
                ty: parse_quote! { i32 },
            },
            ParsedQProperty {
                ident: format_ident!("opaque_property"),
                ty: parse_quote! { UniquePtr<QColor> },
            },
        ];
        let qobject_idents = create_qobjectname();

        let generated =
            generate_cpp_properties(&properties, &qobject_idents, &ParsedCxxMappings::default())
                .unwrap();

        // metaobjects
        assert_eq!(generated.metaobjects.len(), 2);
        assert_str_eq!(generated.metaobjects[0], "Q_PROPERTY(::std::int32_t trivialProperty READ getTrivialProperty WRITE setTrivialProperty NOTIFY trivialPropertyChanged)");
        assert_str_eq!(generated.metaobjects[1], "Q_PROPERTY(::std::unique_ptr<QColor> opaqueProperty READ getOpaqueProperty WRITE setOpaqueProperty NOTIFY opaquePropertyChanged)");

        // methods
        assert_eq!(generated.methods.len(), 8);
        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[0] {
            (header, source)
        } else {
            panic!("Expected pair!")
        };
        assert_str_eq!(header, "::std::int32_t const& getTrivialProperty() const;");
        assert_str_eq!(
            source,
            indoc! {r#"
            ::std::int32_t const&
            MyObject::getTrivialProperty() const
            {
                const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                return getTrivialPropertyWrapper();
            }
            "#}
        );

        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[1] {
            (header, source)
        } else {
            panic!("Expected pair!")
        };
        assert_str_eq!(
            header,
            "Q_SLOT void setTrivialProperty(::std::int32_t const& value);"
        );
        assert_str_eq!(
            source,
            indoc! {r#"
                void
                MyObject::setTrivialProperty(::std::int32_t const& value)
                {
                    const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                    setTrivialPropertyWrapper(value);
                }
                "#}
        );

        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[2] {
            (header, source)
        } else {
            panic!("Expected pair!")
        };
        assert_str_eq!(
            header,
            "::std::unique_ptr<QColor> const& getOpaqueProperty() const;"
        );
        assert_str_eq!(
            source,
            indoc! {r#"
            ::std::unique_ptr<QColor> const&
            MyObject::getOpaqueProperty() const
            {
                const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                return getOpaquePropertyWrapper();
            }
            "#}
        );

        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[3] {
            (header, source)
        } else {
            panic!("Expected pair!")
        };
        assert_str_eq!(
            header,
            "Q_SLOT void setOpaqueProperty(::std::unique_ptr<QColor> const& value);"
        );
        assert_str_eq!(
            source,
            indoc! {r#"
            void
            MyObject::setOpaqueProperty(::std::unique_ptr<QColor> const& value)
            {
                const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                setOpaquePropertyWrapper(value);
            }
            "#}
        );

        let header = if let CppFragment::Header(header) = &generated.methods[4] {
            header
        } else {
            panic!("Expected header!")
        };
        assert_str_eq!(header, "Q_SIGNAL void trivialPropertyChanged();");

        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[5] {
            (header, source)
        } else {
            panic!("Expected Pair")
        };
        assert_str_eq!(
            header,
            "::QMetaObject::Connection trivialPropertyChangedConnect(::rust::Fn<void(MyObject&)> func, ::Qt::ConnectionType type);"
        );
        assert_str_eq!(
            source,
            indoc! {r#"
            ::QMetaObject::Connection
            MyObject::trivialPropertyChangedConnect(::rust::Fn<void(MyObject&)> func, ::Qt::ConnectionType type)
            {
                return ::QObject::connect(this,
                    &MyObject::trivialPropertyChanged,
                    this,
                    [&, func = ::std::move(func)]() {
                        const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                        func(*this);
                    },
                    type);
            }
            "#}
        );

        let header = if let CppFragment::Header(header) = &generated.methods[6] {
            header
        } else {
            panic!("Expected header!")
        };
        assert_str_eq!(header, "Q_SIGNAL void opaquePropertyChanged();");

        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[7] {
            (header, source)
        } else {
            panic!("Expected Pair")
        };
        assert_str_eq!(
            header,
            "::QMetaObject::Connection opaquePropertyChangedConnect(::rust::Fn<void(MyObject&)> func, ::Qt::ConnectionType type);"
        );
        assert_str_eq!(
            source,
            indoc! {r#"
            ::QMetaObject::Connection
            MyObject::opaquePropertyChangedConnect(::rust::Fn<void(MyObject&)> func, ::Qt::ConnectionType type)
            {
                return ::QObject::connect(this,
                    &MyObject::opaquePropertyChanged,
                    this,
                    [&, func = ::std::move(func)]() {
                        const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                        func(*this);
                    },
                    type);
            }
            "#}
        );

        // private methods
        assert_eq!(generated.private_methods.len(), 4);
        let header = if let CppFragment::Header(header) = &generated.private_methods[0] {
            header
        } else {
            panic!("Expected header")
        };
        assert_str_eq!(
            header,
            "::std::int32_t const& getTrivialPropertyWrapper() const noexcept;"
        );

        let header = if let CppFragment::Header(header) = &generated.private_methods[1] {
            header
        } else {
            panic!("Expected header")
        };
        assert_str_eq!(
            header,
            "void setTrivialPropertyWrapper(::std::int32_t value) noexcept;"
        );

        let header = if let CppFragment::Header(header) = &generated.private_methods[2] {
            header
        } else {
            panic!("Expected header")
        };
        assert_str_eq!(
            header,
            "::std::unique_ptr<QColor> const& getOpaquePropertyWrapper() const noexcept;"
        );

        let header = if let CppFragment::Header(header) = &generated.private_methods[3] {
            header
        } else {
            panic!("Expected header")
        };
        assert_str_eq!(
            header,
            "void setOpaquePropertyWrapper(::std::unique_ptr<QColor> value) noexcept;"
        );
    }

    #[test]
    fn test_generate_cpp_properties_mapped_cxx_name() {
        let properties = vec![ParsedQProperty {
            ident: format_ident!("mapped_property"),
            ty: parse_quote! { A1 },
        }];
        let qobject_idents = create_qobjectname();

        let mut cxx_mapping = ParsedCxxMappings::default();
        cxx_mapping
            .cxx_names
            .insert("A".to_owned(), "A1".to_owned());

        let generated =
            generate_cpp_properties(&properties, &qobject_idents, &cxx_mapping).unwrap();

        // metaobjects
        assert_eq!(generated.metaobjects.len(), 1);
        assert_str_eq!(generated.metaobjects[0], "Q_PROPERTY(A1 mappedProperty READ getMappedProperty WRITE setMappedProperty NOTIFY mappedPropertyChanged)");

        // methods
        assert_eq!(generated.methods.len(), 4);
        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[0] {
            (header, source)
        } else {
            panic!("Expected pair!")
        };
        assert_str_eq!(header, "A1 const& getMappedProperty() const;");
        assert_str_eq!(
            source,
            indoc! {r#"
            A1 const&
            MyObject::getMappedProperty() const
            {
                const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                return getMappedPropertyWrapper();
            }
            "#}
        );

        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[1] {
            (header, source)
        } else {
            panic!("Expected pair!")
        };
        assert_str_eq!(header, "Q_SLOT void setMappedProperty(A1 const& value);");
        assert_str_eq!(
            source,
            indoc! {r#"
                void
                MyObject::setMappedProperty(A1 const& value)
                {
                    const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                    setMappedPropertyWrapper(value);
                }
                "#}
        );
        let header = if let CppFragment::Header(header) = &generated.methods[2] {
            header
        } else {
            panic!("Expected header!")
        };
        assert_str_eq!(header, "Q_SIGNAL void mappedPropertyChanged();");

        let (header, source) = if let CppFragment::Pair { header, source } = &generated.methods[3] {
            (header, source)
        } else {
            panic!("Expected Pair")
        };
        assert_str_eq!(
            header,
            "::QMetaObject::Connection mappedPropertyChangedConnect(::rust::Fn<void(MyObject&)> func, ::Qt::ConnectionType type);"
        );
        assert_str_eq!(
            source,
            indoc! {r#"
            ::QMetaObject::Connection
            MyObject::mappedPropertyChangedConnect(::rust::Fn<void(MyObject&)> func, ::Qt::ConnectionType type)
            {
                return ::QObject::connect(this,
                    &MyObject::mappedPropertyChanged,
                    this,
                    [&, func = ::std::move(func)]() {
                        const ::rust::cxxqtlib1::MaybeLockGuard<MyObject> guard(*this);
                        func(*this);
                    },
                    type);
            }
            "#}
        );

        // private methods
        assert_eq!(generated.private_methods.len(), 2);
        let header = if let CppFragment::Header(header) = &generated.private_methods[0] {
            header
        } else {
            panic!("Expected header")
        };
        assert_str_eq!(
            header,
            "A1 const& getMappedPropertyWrapper() const noexcept;"
        );

        let header = if let CppFragment::Header(header) = &generated.private_methods[1] {
            header
        } else {
            panic!("Expected header")
        };
        assert_str_eq!(header, "void setMappedPropertyWrapper(A1 value) noexcept;");
    }
}
