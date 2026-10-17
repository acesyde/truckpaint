## MODIFIED Requirements

### Requirement: Isolated batch outputs
Each invocation that starts a batch SHALL create a unique batch subdirectory under the output parent, defaulting to `dist/`. Intermediates SHALL remain under `target/`, separate from published packages, in a work folder unique to the batch. When the batch succeeds (every requested or discovered vehicle built or skipped as not paintable), its work folder SHALL be deleted after the report is written. When any vehicle fails or the report cannot be written, the work folder SHALL be kept, so converter logs named in failure messages remain available. Work folders of other batches SHALL NOT be touched. Existing batch contents SHALL NOT be overwritten. Vehicles of a batch MAY be built concurrently; each concurrent build SHALL keep its extracted files, converted models and converter logs in its own subfolder of the batch work folder. The published packages, the report content and the order of report entries SHALL be the same as when the vehicles are built one at a time, in trucks-then-trailers id order. Vehicle failures SHALL NOT stop other vehicles. Only fully built and validated packages SHALL be published; failed vehicles and interrupted package writes SHALL leave no partial `.tpv`. The command SHALL return zero only when every requested/discovered vehicle succeeds or is skipped as not paintable and the final report is written; any incomplete batch or output failure SHALL return nonzero.

#### Scenario: Converter failure for one truck
- **WHEN** the converter fails for one vehicle and succeeds for another
- **THEN** the successful package is published, the failed vehicle has no package, both results are reported and the command returns nonzero

#### Scenario: Repeated invocation
- **WHEN** two batches use the same output parent
- **THEN** they use distinct directories and neither overwrites the other's outputs

#### Scenario: Package write failure
- **WHEN** publishing a package fails partway through writing
- **THEN** no partial `.tpv` is left as a published package and the command returns nonzero

#### Scenario: Successful batch cleans its work folder
- **WHEN** every vehicle of a batch is built or skipped as not paintable
- **THEN** the packages and report are published and the batch's work folder under `target/` no longer exists

#### Scenario: Failed batch keeps its work folder
- **WHEN** one vehicle of a batch fails
- **THEN** the batch's work folder, with its converter logs, is kept and the report names it

#### Scenario: Concurrent builds give the same batch
- **WHEN** a batch builds several vehicles concurrently
- **THEN** each package is byte-identical to the package from a batch that builds only that vehicle, and the report lists results in trucks-then-trailers id order
