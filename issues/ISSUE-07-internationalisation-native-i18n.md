# 📌 Issue #07 : Moteur d'Internationalisation Natif (i18n FR / EN)

- **Statut** : 📝 À faire
- **Priorité** : 🟡 Moyenne
- **Composants** : i18n Engine, Localization, Translations Dict
- **Agents Référents** : `.agents/agents/frontend-ui.md`
- **Dépendances** : Issue #01

---

## 🎯 Description du Besoin
MacroForge prend en charge deux langues : le **Français** et l'**Anglais**.
Toutes les chaînes textuelles de l'interface (boutons, infobulles/tooltips, messages de statut, libellés d'actions, descriptions de modales) doivent être gérées par un module i18n natif en Rust, performant, sans allocation dynamique inutile et commutable à la volée.

---

## 📋 Tâches Techniques

1. **Structure des Dictionnaires de Traduction en Rust** :
   - Définir une énumération des langues supportées :
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
     pub enum Language {
         Fr,
         En,
     }
     ```
   - Créer une structure de traduction ou une macro d'accès typé :
     ```rust
     pub struct I18n {
         pub lang: Language,
     }

     impl I18n {
         pub fn t(&self, key: &'static str) -> &'static str {
             match self.lang {
                 Language::Fr => translate_fr(key),
                 Language::En => translate_en(key),
             }
         }
     }
     ```

2. **Migration de l'ensemble des clés depuis `utils.ts`** :
   - Migrer l'intégralité des 100+ entrées de `utils.ts` (boutons, tooltips, modales, types d'actions, messages de succès/erreur).
   - Conserver des clés strictes et sans risque de chaîne manquante à l'exécution.

3. **Changement de Langue Instantané** :
   - Permettre à l'utilisateur de basculer entre Français et Anglais depuis un sélecteur dans l'interface.
   - Sauvegarder le choix de la langue dans les paramètres locaux de l'application.

---

## ✅ Critères d'Acceptation
- [ ] 100% des textes visibles de l'application sont traduits en Français et en Anglais.
- [ ] Le basculement de langue est instantané à l'écran sans nécessiter de redémarrer l'application.
- [ ] Zéro chaîne manquante ou non traduite (`fallback` propre).
