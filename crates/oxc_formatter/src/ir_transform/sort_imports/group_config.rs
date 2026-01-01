use crate::oxfmtrc::CustomGroupDefinition;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Ordering;

/// Parse groups from string-based configuration.
/// If parsing fails (= undefined), it falls back to `Unknown` selector.
pub fn parse_groups_from_strings(string_groups: &Vec<Vec<String>>) -> Vec<Vec<GroupName>> {
    let mut groups = Vec::with_capacity(string_groups.len());
    for group in string_groups {
        let mut parsed_group = Vec::with_capacity(group.len());
        for name in group {
            parsed_group.push(
                GroupName::parse(name).unwrap_or_else(|| GroupName::new(ImportSelector::Unknown)),
            );
        }
        groups.push(parsed_group);
    }
    groups
}

/// Represents a group name pattern for matching imports.
/// A group name consists of 1 selector and N modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupName {
    pub selector: ImportSelector,
    pub modifiers: Vec<ImportModifier>,
}

impl GroupName {
    /// Create a new group name with no modifiers.
    pub fn new(selector: ImportSelector) -> Self {
        Self { selector, modifiers: vec![] }
    }

    /// Create a new group name with one modifier.
    pub fn with_modifier(selector: ImportSelector, modifier: ImportModifier) -> Self {
        Self { selector, modifiers: vec![modifier] }
    }

    /// Check if this is a plain selector (no modifiers).
    pub fn is_plain_selector(&self, selector: ImportSelector) -> bool {
        self.selector == selector && self.modifiers.is_empty()
    }

    /// Parse a group name string into a GroupName.
    ///
    /// Format: `(modifier-)*selector`
    /// Examples:
    /// - "external" -> modifiers: (empty), selector: External
    /// - "type-external" -> modifiers: Type, selector: External
    /// - "value-builtin" -> modifiers: Value, selector: Builtin
    /// - "side-effect-import" -> modifiers: SideEffect, selector: Import
    /// - "side-effect-type-external" -> modifiers: SideEffect, Type, selector: External
    /// - "named-side-effect-type-builtin" -> modifiers: SideEffect, Type, Named, selector: External
    pub fn parse(s: &str) -> Option<Self> {
        // Try to parse as a selector without modifiers first
        if let Some(selector) = ImportSelector::parse(s) {
            return Some(Self { modifiers: vec![], selector });
        }

        // Last part should be the selector
        let selector =
            *ImportSelector::ALL_SELECTORS.iter().find(|selector| s.ends_with(selector.name()))?;

        // The remaining part represents a sequence of modifiers joined by "-".
        // Since modifiers themselves may contain "-",
        // splitting by "-" would be ambiguous.
        // Instead, we iterate over modifiers in a predefined order and check
        // whether they appear in the remaining string.
        // This guarantees the extracted modifiers are already ordered
        // and no additional sorting is required.
        //
        // The trade-off is that this approach may tolerate invalid input,
        // as unmatched or malformed segments are not strictly rejected.
        let mut modifiers = Vec::with_capacity(ImportModifier::ALL_MODIFIERS.len());
        for m in ImportModifier::ALL_MODIFIERS.iter() {
            if s.contains(m.name()) {
                modifiers.push(*m);
            }
        }

        Some(Self { modifiers, selector })
    }

    /// check if this GroupName is one of the possible group names of the given import.
    pub fn is_a_possible_name_of(
        &self,
        selectors: &Vec<ImportSelector>,
        modifiers: &Vec<ImportModifier>,
    ) -> bool {
        selectors.contains(&self.selector) && self.modifiers.iter().all(|m| modifiers.contains(m))
    }
}

impl PartialOrd for GroupName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.selector.partial_cmp(&other.selector) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        let self_modifier_cnt = self.modifiers.len();
        let other_modifier_cnt = self.modifiers.len();
        if self_modifier_cnt > other_modifier_cnt {
            return Some(Ordering::Less);
        } else if self_modifier_cnt < other_modifier_cnt {
            return Some(Ordering::Greater);
        }
        self.modifiers.partial_cmp(&other.modifiers)
    }
}

impl Ord for GroupName {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.selector.cmp(&other.selector) {
            Ordering::Equal => {}
            ord => return ord,
        }
        let self_modifier_cnt = self.modifiers.len();
        let other_modifier_cnt = self.modifiers.len();
        if self_modifier_cnt > other_modifier_cnt {
            return Ordering::Less;
        } else if self_modifier_cnt < other_modifier_cnt {
            return Ordering::Greater;
        }
        self.modifiers.cmp(&other.modifiers)
    }
}

/// Selector types for import categorization.
/// Selectors identify the type or location of an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportSelector {
    /// Type-only imports (`import type { ... }`)
    Type,
    /// Side-effect style imports (CSS, SCSS, etc. without bindings)
    SideEffectStyle,
    /// Side-effect imports (imports without bindings)
    SideEffect,
    /// Style file imports (CSS, SCSS, etc.)
    Style,
    /// Index file imports (`./`, `../`)
    Index,
    /// Sibling module imports (`./foo`)
    Sibling,
    /// Parent module imports (`../foo`)
    Parent,
    /// Subpath imports (package.json imports field, e.g., `#foo`)
    Subpath,
    /// Internal module imports (matching internal patterns like `~/`, `@/`)
    Internal,
    /// Built-in module imports (`node:fs`, `fs`)
    Builtin,
    /// External module imports (from node_modules)
    External,
    /// Catch-all selector
    Import,
    /// Unknown/fallback group
    Unknown,
}

impl ImportSelector {
    /// Parse a string into an ImportSelector.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "type" => Some(Self::Type),
            "side-effect-style" => Some(Self::SideEffectStyle),
            "side-effect" => Some(Self::SideEffect),
            "style" => Some(Self::Style),
            "index" => Some(Self::Index),
            "sibling" => Some(Self::Sibling),
            "parent" => Some(Self::Parent),
            "subpath" => Some(Self::Subpath),
            "internal" => Some(Self::Internal),
            "builtin" => Some(Self::Builtin),
            "external" => Some(Self::External),
            "import" => Some(Self::Import),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    pub const ALL_SELECTORS: &[ImportSelector] = &[
        ImportSelector::Type,
        ImportSelector::SideEffectStyle,
        ImportSelector::SideEffect,
        ImportSelector::Style,
        ImportSelector::Index,
        ImportSelector::Sibling,
        ImportSelector::Parent,
        ImportSelector::Subpath,
        ImportSelector::Internal,
        ImportSelector::Builtin,
        ImportSelector::External,
        ImportSelector::Import,
        ImportSelector::Unknown,
    ];

    pub fn name(&self) -> &str {
        match self {
            ImportSelector::Type => "type",
            ImportSelector::SideEffectStyle => "side-effect-style",
            ImportSelector::SideEffect => "side-effect",
            ImportSelector::Style => "style",
            ImportSelector::Index => "index",
            ImportSelector::Sibling => "sibling",
            ImportSelector::Parent => "parent",
            ImportSelector::Subpath => "subpath",
            ImportSelector::Internal => "internal",
            ImportSelector::Builtin => "builtin",
            ImportSelector::External => "external",
            ImportSelector::Import => "import",
            ImportSelector::Unknown => "unknown",
        }
    }
}

/// Modifier types for import categorization.
/// Modifiers describe characteristics of how an import is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportModifier {
    /// Side-effect imports
    SideEffect,
    /// Type-only imports
    Type,
    /// Value imports (non-type)
    Value,
    /// Require imports
    Require,
    /// Default specifier present
    Default,
    /// Namespace/wildcard specifier present (`* as`)
    Wildcard,
    /// Named specifiers present
    Named,
}

impl ImportModifier {
    /// Parse a string into an ImportModifier.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "side-effect" => Some(Self::SideEffect),
            "type" => Some(Self::Type),
            "value" => Some(Self::Value),
            "require" => Some(Self::Require),
            "default" => Some(Self::Default),
            "wildcard" => Some(Self::Wildcard),
            "named" => Some(Self::Named),
            _ => None,
        }
    }

    pub const ALL_MODIFIERS: &[ImportModifier] = &[
        ImportModifier::SideEffect,
        ImportModifier::Type,
        ImportModifier::Value,
        ImportModifier::Require,
        ImportModifier::Default,
        ImportModifier::Wildcard,
        ImportModifier::Named,
    ];

    pub fn name(&self) -> &str {
        match self {
            ImportModifier::SideEffect => "side-effect",
            ImportModifier::Type => "type",
            ImportModifier::Value => "value",
            ImportModifier::Require => "require",
            ImportModifier::Default => "default",
            ImportModifier::Wildcard => "wildcard",
            ImportModifier::Named => "named",
        }
    }
}

pub struct ImportMetadata<'a> {
    pub source: &'a str,
    pub selectors: Vec<ImportSelector>,
    pub modifiers: Vec<ImportModifier>,
}

pub struct GroupMatcher {
    pub custom_groups: Vec<(CustomGroupDefinition, usize)>,
    pub predefined_groups: Vec<(GroupName, usize)>,
    pub unknown_group_index: usize,
}

impl GroupMatcher {
    pub fn new(groups: &Vec<Vec<String>>, custom_groups: &Vec<CustomGroupDefinition>) -> Self {
        let custom_group_name_set =
            FxHashSet::from_iter(custom_groups.iter().map(|g| g.name.clone()));

        let mut unknown_group_index: Option<usize> = None;

        let mut used_custom_group_index_map = FxHashMap::default();
        let mut predefined_groups = Vec::new();
        for (idx, group_union) in groups.iter().enumerate() {
            for group in group_union.iter() {
                if custom_group_name_set.contains(group) {
                    used_custom_group_index_map.insert(group.to_owned(), idx);
                } else if let Some(group_name) = GroupName::parse(group) {
                    // TODO: should uknown be a ImportSelector?
                    if group_name.is_plain_selector(ImportSelector::Unknown) {
                        unknown_group_index = Some(idx);
                    }
                    predefined_groups.push((group_name, idx));
                }
            }
        }

        let mut used_custom_groups: Vec<(CustomGroupDefinition, usize)> =
            Vec::with_capacity(used_custom_group_index_map.len());
        for custom_group in custom_groups.iter() {
            if let Some(idx) = used_custom_group_index_map.get(&custom_group.name) {
                used_custom_groups.push((custom_group.clone(), *idx));
            }
        }

        predefined_groups.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            custom_groups: used_custom_groups,
            predefined_groups,
            unknown_group_index: unknown_group_index.unwrap_or(groups.len()),
        }
    }

    pub fn compute_group_index(&self, import_metadata: &ImportMetadata) -> usize {
        for (custom_group, index) in self.custom_groups.iter() {
            if custom_group.does_match(import_metadata) {
                return *index;
            }
        }

        for (group_name, index) in self.predefined_groups.iter() {
            if group_name
                .is_a_possible_name_of(&import_metadata.selectors, &import_metadata.modifiers)
            {
                return *index;
            }
        }

        self.unknown_group_index
    }

    pub fn should_regroup_side_effect(&self) -> bool {
        self.predefined_groups
            .iter()
            .any(|(group, _)| group.is_plain_selector(ImportSelector::SideEffect))
    }

    pub fn should_regroup_side_effect_style(&self) -> bool {
        self.predefined_groups
            .iter()
            .any(|(group, _)| group.is_plain_selector(ImportSelector::SideEffectStyle))
    }
}

impl CustomGroupDefinition {
    pub fn does_match(&self, import_metadata: &ImportMetadata) -> bool {
        for rule in self.any_of.iter() {
            if rule.selector.as_ref().is_some_and(|s| {
                ImportSelector::parse(&s)
                    .is_some_and(|selector| !import_metadata.selectors.contains(&selector))
            }) {
                continue;
            }
            if rule.modifiers.as_ref().is_some_and(|modifiers| {
                !modifiers.iter().all(|m| {
                    ImportModifier::parse(m)
                        .is_some_and(|modifier| import_metadata.modifiers.contains(&modifier))
                })
            }) {
                continue;
            }
            if rule
                .element_name_pattern
                .as_ref()
                .is_some_and(|pattern| !import_metadata.source.starts_with(pattern))
            {
                continue;
            }
            return true;
        }
        false
    }
}
