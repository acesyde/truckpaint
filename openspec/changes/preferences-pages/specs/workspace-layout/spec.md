## MODIFIED Requirements

### Requirement: MCP state in the status bar
While the MCP server is on (see the mcp-server capability), the status bar SHALL show, in every space, an item with an icon and a text: "MCP" when no client has called in the last minute, and "MCP · <n> changes" when a client has, where n is the number of write calls applied in the last minute (or "MCP · connected" when they were only reads). Its tooltip SHALL give the server's URL. Clicking it SHALL open Preferences on the AI assistant tab. The item SHALL NOT be shown while the server is off, and its state SHALL NOT rely on color alone.

#### Scenario: Server on, idle
- **WHEN** the server is on and no client has called yet
- **THEN** the status bar shows the MCP item with "MCP"

#### Scenario: Client drawing
- **WHEN** a client has added three shapes in the last minute
- **THEN** the status bar shows "MCP · 3 changes"

#### Scenario: Server off
- **WHEN** the server is off
- **THEN** the status bar shows no MCP item

#### Scenario: Click opens the AI assistant tab
- **WHEN** the server is on and the user clicks the MCP item in the status bar
- **THEN** Preferences opens on the AI assistant tab, with the switch, the address and the port in view
