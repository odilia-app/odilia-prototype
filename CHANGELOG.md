- add afew  log messages
- add log and env_logger as dependencies, to allow some primitive form of logging in stead of relying purely on println since it fills the terminal quickly
- last merge of testing into main, this should fix compilation errors for good
- add the ability to stop speech with the ctrl key
- made sure the screen reader can't panic because of trying to read non-existant accessibles. It will just try to read the attributes of the ones that are actually valid accessibles. If an invalid accessible is incountered, it will sylently be skipped for now
- fix clippy warnings. All unused imports are commented out in stead of removal, in anticipation of later usage
- fix linter warning, add an extra unwrap when creating the speaker global instance
- add basic, messy structural navigation
- Add support for modes, consumption and notification on every keypress.
- Add permission setup
- fix a warning related to static identifier name conventions
- Add read on focus change. Change edition to 2021. Refactor to allow a more user-friendly reporting of controlls and other UI elements

