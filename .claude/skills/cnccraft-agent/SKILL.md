# CncCraft agent guide (skill)

CncCraft is a CNC/CAD editor: you draw shapes on a material board (in millimetres), attach milling
operations (toolpaths) to them, preview the cut in 3D and export G-code. It can be **remote-controlled
for demo recordings**: a script — or an AI agent over MCP — drives the program the way a person would,
with a visible virtual mouse. Every move, click and key press is shown on screen (a large mouse in the
bottom-right corner shows the pressed button, keys appear as keycaps), so the result can be recorded
as a tutorial or TikTok-style video. Captions can be faded in: a standing `text_title` and a changing `text_subtitle`.

Use this guide to write scripts. Play them in CncCraft's script window (Help → Script…, then Play), or
send each line as a `tools/call` to the MCP server at `http://127.0.0.1:8787/` (JSON-RPC over HTTP,
localhost only, no sign-in; it always runs while the app is open).

## Script syntax

- One command per line: `command name=value name=value …`.
- Values: numbers (`120`, `2.5`), words (`Plywood`, `vertical`), quoted text (`"Hello world"`),
  lists (`[1, 2, 3]`), `true` / `false`.
- `# …` or `// …` starts a comment (also at the end of a line). Blank lines are ignored.
- `name = command …` keeps the command's result under `name`. A later line uses it as `$name` (the
  whole result) or `$name.field`, e.g. `$c.id` — also inside lists: `shape_ids=[$c.id, $r.id]`.
- Drawing commands return `{"id": N}`: keep it (`c = draw_circle …`) to cut, move or delete the shape later.
- A line that fails stops the script, and the log says why.

## Coordinates and units

- Shapes, guides and toolpaths use **millimetres on the board**: x from the left edge, y from the
  **bottom** edge (y grows upwards). The board is `width` × `height` from `new_job`.
- The pointer commands (`move_to`, `click`, `drag`, `long_press`, `double_click`) use **screen pixels**
  from the window's top-left; prefer the board-level commands (`drag_shape`, `guide_move`, …), which
  find the right screen place themselves.

## How a script runs

- Each line waits until the visible mouse has finished the previous one, so nothing happens faster
  than a viewer can follow. A change happens only once the mouse has arrived (it moves to a guide's
  handle, then the guide moves).
- `wait seconds=N` holds the scene; `replay_speed speed=N` (0.25–4) makes the mouse slower or faster.
- `text_subtitle` (and `text_title`) do not wait: show the caption, then do the thing it describes.
- Things must be on screen to be clicked: if a shape is outside the view, run
  `view_fit` first. `view mode=3d` shows the board in 3D; the cuts appear after `toolpath_preview`
  (the mouse opens the toolpaths panel on the right and clicks Preview).
- Locked construction lines (`guides_lock_all`) cannot be added, moved or deleted until
  `guides_unlock_all`.
- Read the current situation with `state` (shapes, toolpaths, selection), `guides_list` and `window`
  (window size, panels, view, replay speed, whether the mouse is still busy).

## Commands

Every line of a script is one of these commands. Parameters are `name=value`; required ones are marked. *Returns* is what a later line can use through a variable.

### `agent_md`

Get the agent guide (Markdown): what CncCraft and its script language are for, the syntax, coordinates and pacing rules, every command with its parameters and return value, and example scripts. Read it first to write scripts.

No parameters.

*Returns:* Text: the agent guide as Markdown.

```
agent_md
```

### `state`

Report the current job (size, material), the shapes (id, kind, centre, size in mm) and the toolpaths (index, name, tool, depth). Use it to find shape ids before adding toolpaths.

No parameters.

*Returns:* JSON {"job": {"width", "height", "thickness", "material"}, "selection": [ids], "shapes": [{"id", "kind", "cx", "cy", "width", "height"}], "toolpaths": [{"index", "name", "tool", "cut_depth"}]}.

```
state
```

### `list_materials`

List the material keys available for a job (e.g. for new_job).

No parameters.

*Returns:* JSON {"materials": [keys]}.

```
list_materials
```

### `list_tools`

List the cutters available for the current job's material (name, kind, diameter in mm).

No parameters.

*Returns:* JSON {"material", "tools": [{"name", "kind", "diameter"}]}.

```
list_tools
```

### `window`

Report the window size, whether the cursor is still playing (busy), the MCP port, which side panels are open, the view (2d or 3d), the replay speed and the recording. With width and/or height (logical points), it resizes the window first — set a steady size before recording_start.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `height` | number | no |  | New window height in points (optional; keeps the current height if left out). |
| `width` | number | no |  | New window width in points (optional; keeps the current width if left out). |

*Returns:* JSON {"width", "height", "busy", "port", "panels": {"tools", "milling"}, "view", "replay_speed", "recording": path or null}.

```
window
```

### `new_job`

Start a new job with a size (mm), thickness (mm) and material key (e.g. Softwood, Hardwood, Plywood, MDF, Acrylic, Aluminium).

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `width` | number | yes |  | Width of the material in mm (x). |
| `height` | number | yes |  | Height of the material in mm (y). |
| `material` | string | no |  | Material key from list_materials, e.g. Plywood or MDF. |
| `thickness` | number | no |  | Material thickness in mm (default 18). |

*Returns:* Text "new job".

```
new_job width=10 height=10
```

### `draw_rectangle`

Add a rectangle by its centre (cx, cy in mm) and width/height (mm); optional corner_radius (mm). Returns {"id": ...}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `cx` | number | yes |  | Centre x in mm (from the left edge of the material). |
| `cy` | number | yes |  | Centre y in mm (from the bottom edge of the material). |
| `width` | number | yes |  | Width in mm. |
| `height` | number | yes |  | Height in mm. |
| `corner_radius` | number | no |  | Rounded corners: their radius in mm (default 0, sharp). |

*Returns:* JSON {"id": N} — the new rectangle's shape id.

```
draw_rectangle cx=10 cy=10 width=10 height=10
```

### `draw_circle`

Add a circle by centre (cx, cy in mm) and radius (mm). Returns {"id": ...}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `cx` | number | yes |  | Centre x in mm (from the left edge of the material). |
| `cy` | number | yes |  | Centre y in mm (from the bottom edge of the material). |
| `radius` | number | yes |  | Radius in mm. |

*Returns:* JSON {"id": N} — the new circle's shape id.

```
draw_circle cx=10 cy=10 radius=10
```

### `draw_ellipse`

Add an ellipse by centre (cx, cy in mm) and full width/height (mm). Returns {"id": ...}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `cx` | number | yes |  | Centre x in mm (from the left edge of the material). |
| `cy` | number | yes |  | Centre y in mm (from the bottom edge of the material). |
| `width` | number | yes |  | Full width in mm. |
| `height` | number | yes |  | Full height in mm. |

*Returns:* JSON {"id": N} — the new ellipse's shape id.

```
draw_ellipse cx=10 cy=10 width=10 height=10
```

### `draw_polygon`

Add a regular polygon by centre (cx, cy in mm), radius (mm) and number of sides (>=3). Returns {"id": ...}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `cx` | number | yes |  | Centre x in mm (from the left edge of the material). |
| `cy` | number | yes |  | Centre y in mm (from the bottom edge of the material). |
| `radius` | number | yes |  | Distance from the centre to a corner, in mm. |
| `sides` | number | yes |  | Number of sides (3 or more). |

*Returns:* JSON {"id": N} — the new polygon's shape id.

```
draw_polygon cx=10 cy=10 radius=10 sides=10
```

### `add_text`

Add text by centre (cx, cy in mm), the text and the letter height (mm, default 20). Returns {"id": ...}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `cx` | number | yes |  | Centre x in mm (from the left edge of the material). |
| `cy` | number | yes |  | Centre y in mm (from the bottom edge of the material). |
| `text` | string | yes |  | The text to write. |
| `height` | number | no |  | Letter height in mm (default 20). |

*Returns:* JSON {"id": N} — the new text's shape id.

```
add_text cx=10 cy=10 text="text"
```

### `drag_shape`

Drag a shape with the visible mouse: the cursor goes to the shape, presses, glides and releases so the top-left corner of the shape's outline box lands at (left, top). It is the app's own drag, so the shape snaps to the grid and to construction lines (and attaches to a line it lands on), as by hand. Returns {"id", "from", "to"}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `id` | number | yes |  | The shape's id (from a drawing tool's result or the state tool). |
| `left` | number | yes |  | Where the left edge of the shape should end up, in mm from the left edge of the material. |
| `top` | number | yes |  | Where the top edge of the shape should end up, in mm from the bottom edge of the material. |

*Returns:* JSON {"id", "from": {"left", "top"}, "to": {"left", "top"}} — the place asked for; the shape may snap a little to the grid or a construction line (read it back with state).

```
drag_shape id=$s.id left=10 top=10
```

### `delete_shapes`

Delete shapes by their ids. Returns {"deleted": [...]}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `ids` | array | yes |  | The ids of the shapes to delete. |

*Returns:* JSON {"deleted": [ids]}.

```
delete_shapes ids=[$s.id]
```

### `select`

Select shapes by their ids (clears the previous selection). Returns {"selected": [...]}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `ids` | array | yes |  | The ids of the shapes to select. |

*Returns:* JSON {"selected": [ids]}.

```
select ids=[$s.id]
```

### `set_variable`

Create or change a document variable: a name and an expression (a number or formula). Shapes and toolpaths bound to it follow.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `name` | string | yes |  | The variable's name, e.g. plate_w. |
| `expression` | string | yes |  | A number or formula, e.g. 120 or plate_w/2. |

*Returns:* Text "NAME = EXPRESSION".

```
set_variable name="text" expression="text"
```

### `add_toolpath`

Create a milling toolpath, all through the UI: the visible mouse clicks the shapes to select them (Shift for the extra ones), opens the milling panel if needed, clicks the operation's button, types the depths into the editor's fields when given, and clicks Anwenden. Fields left out keep the material's standard value. All shapes must be on screen (run view_fit first if needed).

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `kind` | string | yes | profile \| cutout \| pocket \| slot \| drill \| vcarve | The milling operation: profile (along the outline), cutout (cut the part free), pocket (clear the inside), slot, drill (holes) or vcarve. |
| `shape_ids` | array | yes |  | The ids of the shapes to cut (from a drawing tool's result or the state tool). |
| `cut_depth` | string | no |  | How deep the cut goes in total (the 'Zieltiefe' field): a number in mm or a variable/formula like platte_d or platte_d/2 (optional). |
| `max_depth_per_pass` | string | no |  | The most the cutter takes per pass (the 'Zustellung' field, 'Bohrtiefe je Hub' for drilling): a number in mm or a formula (optional). |

*Returns:* JSON {"kind", "shape_ids": [ids], "cut_depth", "max_depth_per_pass"} — the toolpath is computed once the mouse has clicked Anwenden; read it back with state.

```
add_toolpath kind=profile shape_ids=[$s.id]
```

### `delete_toolpath`

Delete a toolpath by its index in the list (0-based).

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `index` | number | yes |  | Position of the toolpath in the list, starting at 0 (see the state tool). |

*Returns:* Text "deleted NAME".

```
delete_toolpath index=10
```

### `guides_list`

List the construction lines (ruler guides): id, axis (vertical = an X value, horizontal = a Y value), offset in mm, formula and attached shape ids, plus whether the lines are locked and the rulers shown.

No parameters.

*Returns:* JSON {"rulers_shown", "locked", "guides": [{"id", "axis", "offset", "expr", "attached": [shape ids]}]}.

```
guides_list
```

### `guide_add`

Add a construction line (ruler guide): axis "vertical" (an X value, measured from the left edge) or "horizontal" (a Y value, from the bottom), at offset mm. Turns the rulers on if needed. Returns {"id": ...}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `axis` | string | yes | vertical \| horizontal | vertical: a line at an x value; horizontal: a line at a y value. |
| `offset` | number | yes |  | Distance in mm from the left edge (vertical) or the bottom edge (horizontal). |

*Returns:* JSON {"id": N} — the new construction line's id.

```
guide_add axis=vertical offset=10
```

### `guide_move`

Move a construction line (from guides_list) to a new distance (mm). The visible mouse drags it on the ruler; any attached shapes follow. Returns {"id": ...}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `id` | number | yes |  | The construction line's id (from guide_add or guides_list). |
| `offset` | number | yes |  | The new distance in mm from the left edge (vertical) or the bottom edge (horizontal). |

*Returns:* JSON {"id": N}.

```
guide_move id=$s.id offset=10
```

### `guide_delete`

Delete a construction line by its id (from guides_list); its shapes stay put. Returns {"deleted": id}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `id` | number | yes |  | The construction line's id (from guide_add or guides_list). |

*Returns:* JSON {"deleted": id}.

```
guide_delete id=$s.id
```

### `guides_delete_all`

Delete every construction line: the mouse glides from one guide's handle to the next and removes each. Shapes stay put. Returns {"deleted": [ids...]}.

No parameters.

*Returns:* JSON {"deleted": [ids]}.

```
guides_delete_all
```

### `guides_lock_all`

Lock every construction line (the mouse clicks the lock in the rulers' corner). Locked lines can't be added, moved, edited or deleted; shapes still snap to them.

No parameters.

*Returns:* Text "locking all construction lines".

```
guides_lock_all
```

### `guides_unlock_all`

Unlock every construction line (the mouse clicks the lock in the rulers' corner), so they can be moved, edited and deleted again.

No parameters.

*Returns:* Text "unlocking all construction lines".

```
guides_unlock_all
```

### `save_project`

Save the project to an absolute .cnccraft path.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `path` | string | yes |  | Absolute path of the .cnccraft project file. |

*Returns:* Text "saved PATH".

```
save_project path=/tmp/out
```

### `export_gcode`

Write the visible toolpaths as a G-code file to an absolute path.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `path` | string | yes |  | Absolute path of the G-code file to write. |

*Returns:* Text saying which file was written.

```
export_gcode path=/tmp/out
```

### `view`

Switch to the 2D or 3D view: the visible mouse clicks the 2D/3D toggle in the canvas bar (nothing is clicked if the view already is the one asked for). Returns {"mode", "changed"}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `mode` | string | yes | 2d \| 3d | 2d: the drawing; 3d: the board in 3D (the cuts show once the preview runs: toolpath_preview). |

*Returns:* JSON {"mode": "2d"|"3d", "changed": true|false}.

```
view mode=2d
```

### `panel_tools_open`

Open the tool column on the left: the visible mouse moves to the panel's fold button (the chevron) and clicks it. Nothing is clicked if it already is open. Returns {"panel", "open", "changed"}.

No parameters.

*Returns:* JSON {"panel": "tools"|"milling", "open": true|false, "changed": true|false}.

```
panel_tools_open
```

### `panel_tools_close`

Close the tool column on the left: the visible mouse moves to the panel's fold button (the chevron) and clicks it. Nothing is clicked if it already is closed. Returns {"panel", "open", "changed"}.

No parameters.

*Returns:* JSON {"panel": "tools"|"milling", "open": true|false, "changed": true|false}.

```
panel_tools_close
```

### `panel_milling_open`

Open the milling (toolpath) panel on the right: the visible mouse moves to the panel's fold button (the chevron) and clicks it. Nothing is clicked if it already is open. Returns {"panel", "open", "changed"}.

No parameters.

*Returns:* JSON {"panel": "tools"|"milling", "open": true|false, "changed": true|false}.

```
panel_milling_open
```

### `panel_milling_close`

Close the milling (toolpath) panel on the right: the visible mouse moves to the panel's fold button (the chevron) and clicks it. Nothing is clicked if it already is closed. Returns {"panel", "open", "changed"}.

No parameters.

*Returns:* JSON {"panel": "tools"|"milling", "open": true|false, "changed": true|false}.

```
panel_milling_close
```

### `demo_window`

Open or close the in-app Demo & remote-control window.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `open` | boolean | no |  | true opens the window, false closes it (default true). |

*Returns:* Text "demo window open" or "demo window closed".

```
demo_window
```

### `script_window`

Open or close the script window (its own OS window for writing and playing a script of commands).

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `open` | boolean | no |  | true opens the window, false closes it (default true). |

*Returns:* Text "script window open" or "script window closed".

```
script_window
```

### `screenshot`

Save a PNG of the window to an absolute path (does not close the app).

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `path` | string | yes |  | Absolute path of the PNG file to write. |

*Returns:* Text "saving to PATH" (the PNG is written on the next frame).

```
screenshot path=/tmp/out
```

### `file_new`

New job — The mouse opens the File menu and clicks "New job". Keys: ⌘N.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
file_new
```

### `file_open`

Open… — The mouse opens the File menu and clicks "Open…". Keys: ⌘O.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
file_open
```

### `file_save`

Save — The mouse opens the File menu and clicks "Save". Keys: ⌘S.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
file_save
```

### `file_save_as`

Save as… — The mouse opens the File menu and clicks "Save as…". Keys: ⇧⌘S.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
file_save_as
```

### `file_export_toolpaths`

Save toolpaths… — The mouse opens the File menu and clicks "Save toolpaths…".

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
file_export_toolpaths
```

### `file_start_page`

Start page — The mouse opens the File menu and clicks "Start page".

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
file_start_page
```

### `edit_undo`

Undo — The mouse opens the Edit menu and clicks "Undo". Keys: ⌘Z.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_undo
```

### `edit_redo`

Redo — The mouse opens the Edit menu and clicks "Redo". Keys: ⇧⌘Z.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_redo
```

### `edit_cut`

Cut — The mouse opens the Edit menu and clicks "Cut". Keys: ⌘X.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_cut
```

### `edit_copy`

Copy — The mouse opens the Edit menu and clicks "Copy". Keys: ⌘C.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_copy
```

### `edit_paste`

Paste — The mouse opens the Edit menu and clicks "Paste". Keys: ⌘V.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_paste
```

### `edit_duplicate`

Duplicate — The mouse opens the Edit menu and clicks "Duplicate". Keys: ⌘D.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_duplicate
```

### `edit_delete`

Delete — The mouse opens the Edit menu and clicks "Delete". Keys: ⌫.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_delete
```

### `edit_group`

Group — The mouse opens the Edit menu and clicks "Group". Keys: ⌘G.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_group
```

### `edit_ungroup`

Ungroup — The mouse opens the Edit menu and clicks "Ungroup". Keys: ⇧⌘G.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_ungroup
```

### `edit_select_all`

Select all — The mouse opens the Edit menu and clicks "Select all". Keys: ⌘A.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_select_all
```

### `edit_node_edit`

Edit nodes — The mouse opens the Edit menu and clicks "Edit nodes". Keys: N.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_node_edit
```

### `edit_job_setup`

Set up job… — The mouse opens the Edit menu and clicks "Set up job…".

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_job_setup
```

### `edit_cancel`

Cancel — The mouse opens the Edit menu and clicks "Cancel". Keys: Esc.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
edit_cancel
```

### `tool_select`

Select — Tools › Select. Keys: S.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
tool_select
```

### `tool_line`

Draw line — Tools › Draw line. Keys: L.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
tool_line
```

### `tool_spline`

Draw spline — Tools › Draw spline. Keys: P.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
tool_spline
```

### `tool_rectangle`

Draw rectangle — Tools › Draw rectangle. Keys: R.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
tool_rectangle
```

### `tool_circle`

Draw circle — Tools › Draw circle. Keys: C.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
tool_circle
```

### `tool_offset`

Offset vectors — Tools › Offset vectors. Keys: O.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
tool_offset
```

### `tool_dimension`

Bemaßen — Tools › Bemaßen. Keys: D.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
tool_dimension
```

### `tool_align`

Waagrecht/Senkrecht — Tools › Waagrecht/Senkrecht. Keys: H.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
tool_align
```

### `view_2d`

2D view — The mouse opens the View menu and clicks "2D view". Keys: F2.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
view_2d
```

### `view_3d`

3D view — The mouse opens the View menu and clicks "3D view". Keys: F3.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
view_3d
```

### `view_fit`

Fit view — The mouse opens the View menu and clicks "Fit view". Keys: F4.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
view_fit
```

### `view_toggle_dimensions`

Show dimensions — The mouse opens the View menu and clicks "Show dimensions".

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
view_toggle_dimensions
```

### `toolpath_profile`

Profile — The mouse opens the Toolpaths menu and clicks "Profile". Opens the Profil toolpath editor for the selected shapes.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_profile
```

### `toolpath_pocket`

Pocket — The mouse opens the Toolpaths menu and clicks "Pocket". Opens the Tasche (pocket) toolpath editor.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_pocket
```

### `toolpath_drill`

Drilling — The mouse opens the Toolpaths menu and clicks "Drilling". Opens the Bohren (drill) toolpath editor.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_drill
```

### `toolpath_vcarve`

V-carving — The mouse opens the Toolpaths menu and clicks "V-carving". Opens the V-Carving toolpath editor.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_vcarve
```

### `toolpath_cutout`

Cutout — The mouse opens the Toolpaths menu and clicks "Cutout". Opens the Ausschnitt (cutout) toolpath editor.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_cutout
```

### `toolpath_recompute_all`

Recalculate all — The mouse opens the Toolpaths menu and clicks "Recalculate all".

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_recompute_all
```

### `toolpath_preview`

Preview — Toolpaths panel › Preview. The mouse opens the toolpaths panel on the right if needed and clicks Preview: the finished cut in 3D.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_preview
```

### `toolpath_simulate`

Simulate — Toolpaths panel › Simulate. The mouse opens the toolpaths panel on the right if needed and clicks Simulate: the cut animated in 3D.

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_simulate
```

### `toolpath_gcode_motion_on`

Show G-code motion — Toolpaths panel › Show G-code motion. The mouse clicks the "G-code motion in 3D" checkbox so the machine's moves are drawn over the 3D view (does nothing if already on).

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_gcode_motion_on
```

### `toolpath_gcode_motion_off`

Hide G-code motion — Toolpaths panel › Hide G-code motion. The mouse clicks the "G-code motion in 3D" checkbox so the moves are hidden (does nothing if already off).

No parameters.

*Returns:* Text saying what is being done, e.g. "opening Edit › Undo in the menu" (the mouse opens the menu and clicks the item) or "clicking Preview in the toolpaths panel".

```
toolpath_gcode_motion_off
```

### `cursor`

Show or hide the virtual mouse cursor.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `on` | boolean | yes |  | true shows the cursor, false hides it. |

*Returns:* Text "cursor on" or "cursor off".

```
cursor on=true
```

### `move_to`

Glide the cursor to a screen point (pixels from the top-left of the window).

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `x` | number | yes |  | Screen x in pixels from the left of the window. |
| `y` | number | yes |  | Screen y in pixels from the top of the window. |

*Returns:* Text "moving"; the cursor glides there.

```
move_to x=10 y=10
```

### `click`

Click at a screen point with the left (default) or right button.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `x` | number | yes |  | Screen x in pixels from the left of the window. |
| `y` | number | yes |  | Screen y in pixels from the top of the window. |
| `button` | string | no | left \| right | Which mouse button (default left). |

*Returns:* Text "clicked"; the click plays on the visible cursor.

```
click x=10 y=10
```

### `double_click`

Double-click at a screen point.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `x` | number | yes |  | Screen x in pixels from the left of the window. |
| `y` | number | yes |  | Screen y in pixels from the top of the window. |

*Returns:* Text "double-clicked"; it plays on the visible cursor.

```
double_click x=10 y=10
```

### `long_press`

Long-press at a screen point (to open a value for editing, like a person holding the mouse).

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `x` | number | yes |  | Screen x in pixels from the left of the window. |
| `y` | number | yes |  | Screen y in pixels from the top of the window. |

*Returns:* Text "long-pressed"; it plays on the visible cursor.

```
long_press x=10 y=10
```

### `drag`

Press at (x1,y1), drag to (x2,y2) and release.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `x1` | number | yes |  | Start x in screen pixels (where the button goes down). |
| `y1` | number | yes |  | Start y in screen pixels. |
| `x2` | number | yes |  | End x in screen pixels (where the button is released). |
| `y2` | number | yes |  | End y in screen pixels. |

*Returns:* Text "dragged"; it plays on the visible cursor.

```
drag x1=10 y1=10 x2=10 y2=10
```

### `type_text`

Type text into the focused field.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `text` | string | yes |  | The text to type into the focused field. |

*Returns:* Text "typed".

```
type_text text="text"
```

### `press_key`

Press a key (Enter, Escape, Tab, Delete, Backspace, Space, arrows, or a single letter/digit) with optional modifiers cmd/shift/alt/ctrl.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `key` | string | yes |  | The key: Enter, Escape, Tab, Delete, Backspace, Space, Up, Down, Left, Right, or a single letter/digit. |
| `alt` | boolean | no |  | Hold Alt/Option. |
| `cmd` | boolean | no |  | Hold ⌘ (Ctrl on Windows/Linux). |
| `ctrl` | boolean | no |  | Hold Ctrl (treated like cmd). |
| `shift` | boolean | no |  | Hold Shift. |

*Returns:* Text "key"; the keys show as keycaps next to the big mouse.

```
press_key key="text"
```

### `wait`

Pause for a number of seconds (a beat for the viewer).

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `seconds` | number | yes |  | How long to pause, in seconds (decimals allowed). |

*Returns:* Text "waiting".

```
wait seconds=10
```

### `replay_speed`

Set how fast the visible cursor plays its moves, clicks and pauses: 1 is normal, 2 twice as fast, 0.5 half as fast (0.25 to 4). A long press is never made shorter than it must be. Without speed, it reports the current one. Returns {"speed": ...}.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `speed` | number | no |  | The replay speed: 1 normal, 2 twice as fast, 0.5 half as fast (kept between 0.25 and 4). |

*Returns:* JSON {"speed": N} — the speed now in use.

```
replay_speed
```

### `text_title`

Show a standing title over the window (e.g. the app's name), kept until it is replaced or cleared, like the text in TikTok or tutorial videos: wrapped on a rounded box, fading in and out. An empty text removes it. It does not wait; add wait to hold the scene. Defaults: top centre, 40 pt, no time limit. Headline and subtitle are independent — both can show at once.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `text` | string | yes |  | The caption. An empty text takes the current one away. |
| `bg_color` | string | no |  | Box colour as #RRGGBB, #RRGGBBAA or r,g,b,a (default black at 70%). Also accepted as bg-color. |
| `color` | string | no |  | Text colour as #RRGGBB, #RRGGBBAA or r,g,b,a with 0-255 (default white). |
| `duration` | number | no |  | How long it shows, in seconds (0 keeps it until replaced). |
| `font` | number | no |  | Font size in points. |
| `horizontal` | string | no | left \| center \| right | Where it sits horizontally, and how its lines align (default center). |
| `opacity` | number | no |  | Overall opacity from 0 to 1 (default 1). |
| `vertical` | string | no | top \| center \| bottom | Where it sits vertically. |

*Returns:* JSON {"shown": true, "duration": seconds}, or {"shown": false} when the text was empty (the caption is taken away).

```
text_title text="text"
```

### `text_subtitle`

Show the subtitle that says what is happening, replaced each time it is called, like the text in TikTok or tutorial videos: wrapped on a rounded box, fading in and out. An empty text removes it. It does not wait; add wait to hold the scene. Defaults: bottom centre, 28 pt, 3 seconds. Headline and subtitle are independent — both can show at once.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `text` | string | yes |  | The caption. An empty text takes the current one away. |
| `bg_color` | string | no |  | Box colour as #RRGGBB, #RRGGBBAA or r,g,b,a (default black at 70%). Also accepted as bg-color. |
| `color` | string | no |  | Text colour as #RRGGBB, #RRGGBBAA or r,g,b,a with 0-255 (default white). |
| `duration` | number | no |  | How long it shows, in seconds (0 keeps it until replaced). |
| `font` | number | no |  | Font size in points. |
| `horizontal` | string | no | left \| center \| right | Where it sits horizontally, and how its lines align (default center). |
| `opacity` | number | no |  | Overall opacity from 0 to 1 (default 1). |
| `vertical` | string | no | top \| center \| bottom | Where it sits vertically. |

*Returns:* JSON {"shown": true, "duration": seconds}, or {"shown": false} when the text was empty (the caption is taken away).

```
text_subtitle text="text"
```

### `recording_start`

Start recording the window to an .mp4 video (H.264, encoded inside the app). Everything shown is captured, including the virtual mouse and captions. Stop it with recording_stop. Set the window size with window first, since the video keeps the size it starts with.

| Parameter | Type | Required | Values | Meaning |
|---|---|---|---|---|
| `file` | string | yes |  | Absolute path of the .mp4 file to write. |
| `fps` | number | no |  | Frames per second (default 30, 1 to 60). |
| `max_side` | number | no |  | Largest side of the video in pixels (default 1920); a bigger window is scaled down by a whole factor. |

*Returns:* JSON {"recording": path, "fps": N}.

```
recording_start file="text"
```

### `recording_stop`

Stop the video recording and close the .mp4 file. Returns where it was written and how long it is.

No parameters.

*Returns:* JSON {"file", "frames", "seconds", "width", "height"}.

```
recording_stop
```

## Example scripts

### Draw a part, cut it out and show it in 3D

```
# a 300 x 200 mm plywood board
replay_speed speed=1
new_job width=300 height=200 thickness=18 material=Plywood
text_title text="CncCraft — CAM for the hobbyist"
text_subtitle text="A round pocket in two steps" duration=3
c = draw_circle cx=150 cy=100 radius=40
add_toolpath kind=pocket shape_ids=[$c.id]
r = draw_rectangle cx=150 cy=100 width=240 height=160 corner_radius=10
add_toolpath kind=cutout shape_ids=[$r.id]
view mode=3d
toolpath_preview
wait seconds=2
```

### Snap a shape onto construction lines

```
new_job width=300 height=200
g = guide_add axis=vertical offset=50
h = guide_add axis=horizontal offset=150
s = draw_rectangle cx=150 cy=100 width=60 height=40
text_subtitle text="Drag it onto the guides"
drag_shape id=$s.id left=51 top=149     # lands on x=50 / y=150 and attaches
wait seconds=1
guide_move id=$g.id offset=80           # the attached shape follows the line
guides_lock_all
```

### Use the menus and panels like a person

```
panel_milling_open
edit_select_all
edit_duplicate
press_key key=z cmd=true                 # undo, shown as keycaps
panel_tools_close
text_subtitle text=""               # remove the subtitle
```
