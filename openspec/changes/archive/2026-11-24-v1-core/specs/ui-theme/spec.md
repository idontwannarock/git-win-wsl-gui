## ADDED Requirements

### Requirement: Console dark theme
The system SHALL use one dark theme based on the Hallmark "Console" tokens: paper `oklch(17% 0.008 250)`, paper-2 `oklch(21% 0.009 250)`, ink `oklch(95% 0.005 250)`, muted `oklch(71% 0.01 250)`, rule `oklch(31% 0.01 250)`, accent `oklch(82% 0.15 80)`, ok `oklch(76% 0.15 150)`, bad `oklch(70% 0.17 25)`, card radius 10 px. Code and hashes SHALL use a monospace font. All other UI text SHALL use one sans-serif font.

#### Scenario: Tokens in one place
- **WHEN** a view needs a color or radius
- **THEN** it reads the value from the theme tokens and not from a literal

### Requirement: Contrast floor
Body text and code SHALL have a contrast ratio of at least 4.5:1 against its background. UI borders and focus rings SHALL have at least 3:1.

#### Scenario: Token check
- **WHEN** the theme test runs
- **THEN** every text/background pair used by the app meets its ratio

### Requirement: Keyboard access
Every action in v1 SHALL be reachable by keyboard. The focused element SHALL show a visible focus ring in the accent color. The Tab order SHALL follow the visual order.

#### Scenario: Stage by keyboard
- **WHEN** focus is on a file row and the user presses Space
- **THEN** the file is staged or unstaged

### Requirement: Reduced motion
The system SHALL use no animation that is not needed to explain a state change. The system SHALL respect the Windows "show animations" setting by turning off all animation when it is off.

#### Scenario: Animations off
- **WHEN** Windows "show animations" is off
- **THEN** the app plays no transition

### Requirement: No decorative AI-style ornament
The UI SHALL NOT use gradient text, glow effects, decorative icons without a function, or tracked-capital eyebrow labels. Every visual element SHALL carry information or an action.

#### Scenario: Design review
- **WHEN** a reviewer checks the UI against this list
- **THEN** no listed ornament is present
