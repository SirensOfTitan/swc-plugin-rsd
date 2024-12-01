use std::collections::HashMap;

use swc_atoms::JsWord;
use swc_common::DUMMY_SP;
use swc_core::ecma::utils::private_ident;
use swc_core::ecma::visit::visit_mut_pass;
use swc_core::{ecma::ast::*, ecma::visit::VisitMut};
use swc_ecma_visit::VisitMutWith;

pub fn react_strict_dom_plugin() -> impl Pass {
    visit_mut_pass(ReactStrictDomPlugin {
        ..Default::default()
    })
}

#[derive(Default)]
struct ReactStrictDomPlugin {
    /// Imports of react-strict-dom in current module.
    rsd_imports: Vec<ImportDecl>,

    /// Used to name injected identifiers.
    cnt: usize,

    /// The name of the added runtime default styles import.
    default_styles: Option<JsWord>,

    /// Then name of the added runtime resolve styles import.
    resolve_style: Option<JsWord>,
}

impl ReactStrictDomPlugin {
    fn next_variable_id(&mut self, prefix: &str) -> JsWord {
        self.cnt += 1;
        format!("$_{}_{}", prefix, self.cnt).into()
    }

    fn rsd_html_element<'a>(
        &'a self,
        jsx_element_name: &'a JSXElementName,
    ) -> Option<&'a JSXMemberExpr> {
        let JSXElementName::JSXMemberExpr(jsx_member_expr) = jsx_element_name else {
            return None;
        };

        let Some(object_name) = jsx_member_expr.obj.as_ident() else {
            return None;
        };

        let is_html_element = self
            .rsd_imports
            .iter()
            .flat_map(|x| {
                x.specifiers
                    .iter()
                    .filter_map(|s| match s {
                        ImportSpecifier::Named(inner) => {
                            let export_name = inner
                                .imported
                                .as_ref()
                                .map(|x| x.atom())
                                .unwrap_or(&inner.local.sym);

                            if export_name.eq("html") {
                                Some(inner)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .any(|decl| decl.local.sym.eq(object_name.sym.as_str()));

        if is_html_element {
            Some(jsx_member_expr)
        } else {
            None
        }
    }

    fn style_prop_value(&self, value: Option<JSXAttrValue>) -> Vec<Option<ExprOrSpread>> {
        let Some(value) = value else {
            return vec![];
        };

        let JSXAttrValue::JSXExprContainer(value) = value else {
            return vec![];
        };

        let JSXExpr::Expr(inner_expr) = value.expr else {
            return vec![];
        };

        if let Expr::Array(lit) = *inner_expr {
            lit.elems
        } else {
            vec![Some(ExprOrSpread {
                spread: None,
                expr: inner_expr,
            })]
        }
    }

    /// Updates the attributes of a JSXOpeningElement based on the current plugin state.
    ///
    /// Returns attributes that need to be added to the element.
    fn update_jsx_attributes(
        &mut self,
        jsx_opening_element: &mut JSXOpeningElement,
    ) -> Vec<JSXAttrOrSpread> {
        let Some(rsd_html_element) = self.rsd_html_element(&jsx_opening_element.name) else {
            return vec![];
        };

        let element_name = rsd_html_element.prop.sym.clone();
        jsx_opening_element.name = JSXElementName::Ident(private_ident!(element_name.as_str()));

        let mut keyed_attrs = jsx_opening_element
            .attrs
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, attr)| match attr {
                JSXAttrOrSpread::JSXAttr(jsx_attr) => {
                    if let JSXAttrName::Ident(attr_ident) = &jsx_attr.name {
                        Some((attr_ident.sym.clone(), (idx, jsx_attr)))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        if let Some((_, for_attr)) = keyed_attrs.remove(&"for".into()) {
            for_attr.name = JSXAttrName::Ident("htmlFor".into());
        }

        if let Some((_, role_attr)) = keyed_attrs.remove(&"role".into()) {
            let is_role_none = role_attr
                .value
                .as_mut()
                .map(|x| matches!(x, JSXAttrValue::Lit(Lit::Str(x)) if x.value.eq("none")))
                .unwrap_or(false);

            if is_role_none {
                role_attr.value = Some(JSXAttrValue::Lit("presentation".into()));
            }
        }

        let mut added_attrs = vec![];

        // Add dir="auto" to input and textarea elements, if not already set.
        if !keyed_attrs.contains_key(&"dir".into())
            && (element_name.eq("input") || element_name.eq("textarea"))
        {
            added_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident("dir".into()),
                value: Some(JSXAttrValue::Lit("auto".into())),
            }));
        }

        if !keyed_attrs.contains_key(&"type".into()) && element_name.eq("button") {
            added_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident("type".into()),
                value: Some(JSXAttrValue::Lit("button".into())),
            }));
        }

        let default_styles_expr = MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(
                self.default_styles.as_ref().unwrap().clone().into(),
            )),
            prop: MemberProp::Ident(element_name.clone().into()),
        };

        let style_attr = keyed_attrs
            .get(&"style".into())
            .and_then(|(_, x)| x.value.clone());
        let style_args = self.style_prop_value(style_attr);

        let args = [
            vec![Some(ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Member(default_styles_expr)),
            })],
            style_args,
        ]
        .concat()
        .into_iter()
        .flatten()
        .collect();

        let style_spread = JSXAttrOrSpread::SpreadElement(SpreadElement {
            dot3_token: DUMMY_SP,
            expr: Box::new(Expr::Call(CallExpr {
                args,
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(
                    self.resolve_style.as_ref().unwrap().clone().into(),
                ))),
                ..Default::default()
            })),
        });

        if let Some((idx, _)) = keyed_attrs.remove(&"style".into()) {
            jsx_opening_element.attrs[idx] = style_spread;
        } else {
            added_attrs.push(style_spread);
        }

        added_attrs
    }
}

const RSD_RUNTIME_IMPORT: &str = "react-strict-dom/runtime";

impl VisitMut for ReactStrictDomPlugin {
    fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
        if !import_decl.src.value.eq("react-strict-dom") {
            return;
        }

        self.rsd_imports.push(import_decl.to_owned());
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        self.default_styles = Some(self.next_variable_id("defaultStyles"));
        self.resolve_style = Some(self.next_variable_id("resolveStyle"));

        module.visit_mut_children_with(self);

        if self.rsd_imports.is_empty() {
            return;
        }

        let runtime_import = ImportDecl {
            src: Box::new(RSD_RUNTIME_IMPORT.into()),
            span: DUMMY_SP,
            specifiers: vec![
                ImportSpecifier::Named(ImportNamedSpecifier {
                    local: self.default_styles.as_ref().unwrap().clone().into(),
                    imported: Some(ModuleExportName::Ident(private_ident!("defaultStyles"))),
                    span: DUMMY_SP,
                    is_type_only: false,
                }),
                ImportSpecifier::Named(ImportNamedSpecifier {
                    local: self.resolve_style.as_ref().unwrap().clone().into(),
                    imported: Some(ModuleExportName::Ident(private_ident!("resolveStyle"))),
                    span: DUMMY_SP,
                    is_type_only: false,
                }),
            ],
            type_only: false,
            phase: ImportPhase::Evaluation,
            with: None,
        };

        module.body.insert(0, runtime_import.into());
    }

    fn visit_mut_jsx_opening_element(&mut self, jsx_opening_element: &mut JSXOpeningElement) {
        let added_attrs = self.update_jsx_attributes(jsx_opening_element);

        if added_attrs.is_empty() {
            return;
        }

        jsx_opening_element.attrs.extend(added_attrs);
    }

    fn visit_mut_jsx_closing_element(&mut self, jsx_closing_element: &mut JSXClosingElement) {
        let Some(rsd_html_element) = self.rsd_html_element(&jsx_closing_element.name) else {
            return;
        };

        jsx_closing_element.name =
            JSXElementName::Ident(private_ident!(rsd_html_element.prop.sym.as_str()));
    }
}

#[cfg(test)]
mod test {
    use swc_core::ecma::parser::{Syntax, TsSyntax};
    use swc_core::ecma::transforms::testing::test_inline;

    use super::react_strict_dom_plugin;

    test_inline!(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..TsSyntax::default()
        }),
        |_| react_strict_dom_plugin(),
        files_that_use_rsd_should_have_runtime_import,
        r#"import {css, html as h} from "react-strict-dom";

        function App() {
          return (
            <h.div for="asdf" role="none" style={styles.foo}>
              foo
            </h.div>
          );
        }"#,
        r#"import { defaultStyles as $_defaultStyles_1, resolveStyle as $_resolveStyle_2 } from "react-strict-dom/runtime";
        import {css, html as h} from "react-strict-dom";

        function App() {
          return <div htmlFor="asdf" role="presentation" {...$_resolveStyle_2($_defaultStyles_1.div, styles.foo)}>
              foo
            </div>;
        }"#
    );

    test_inline!(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..TsSyntax::default()
        }),
        |_| react_strict_dom_plugin(),
        test_changed_special_attributes,
        r#"import {css, html} from "react-strict-dom";

        function App() {
          return (
            <>
              <html.textarea />
              <html.button style={[styles.bar, styles.foo]}>
                hello
              </html.button>
            </>
          );
        }"#,
        r#"import { defaultStyles as $_defaultStyles_1, resolveStyle as $_resolveStyle_2 } from "react-strict-dom/runtime";
        import {css, html} from "react-strict-dom";

        function App() {
          return <>
              <textarea dir="auto" {...$_resolveStyle_2($_defaultStyles_1.textarea)} />
              <button {...$_resolveStyle_2($_defaultStyles_1.button, styles.bar, styles.foo)} type="button">
                hello
              </button>
            </>;
        }"#
    );
}
