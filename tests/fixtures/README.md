# Test Fixtures Layout

This directory is organized by test purpose.

- `solver/`: CCX regression corpus (`.inp` inputs and companion reference/support files).
- `viewer/`: CGX viewer smoke inputs (currently `.frd` files).
- `nastran/`: Nastran reader fixtures.

Notes:
- Keep companion files in the same folder as the owning `.inp` deck so relative includes continue to work.
- Add new CCX solver fixtures to `solver/`.
- Add new CGX viewer fixtures to `viewer/`.
