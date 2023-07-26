# Klipper GCode Preprocessor for per-feature acceleration control

This is a Rust port of the `acceleration.py` script by VintageGriffin that adds
support for a few more Slicers in the process. This post-processor allows
setting separate acceleration, accel-to-decel and square corner velocity values
for print moves per feature type as well as for travel moves.

The following slicers are currently supported:

* SuperSlicer
* PrusaSlicer
* Orcaslicer

## Installation and usage

To configure a supported Slicer to automatically run this script against all
GCode files that it generates the following steps need to be performed:

1. Grab the latest released binary for your operating system from the
   [releases](https://github.com/mjonuschat/acceleration-control/releases) page.
2. Copy the binary to a permanent home, for example, `/Users/myuser/Documents/3D-Printing/Tools` on a Mac.
   * On macOS, you need to remove the "Quarantine Flag" from the binary as it has not been signed by Apple.  
     To do that run `sudo xattr -r -d com.apple.quarantine /path/to/acceleration-control`.
3. Add `/Users/myuser/Documents/3D-Printing/Tools/acceleration-control;` to the
   Slicer post-processing options:
    * In SuperSlicer: Print Settings > Output Options > Post Processing Scripts
    * In PrusaSlicer: Print Settings > Output Options > Post Processing Scripts
    * In OrcaSlicer: Process > Other > Post Processing Scripts
4. If you have previously done acceleration control via the custom code in the
   "between extrusion role change" G-Code section (in the SuperSlicer under
   Printer Settings > Custom GCode), remove it from there. This post-processing
   script will inject all the necessary acceleration control statements.
5. (Optionally) Disable advanced acceleration control (set all values to 0) in
   the Print Settings > Speed section if your Slicer supports it. This script
   will automatically remove any Marlin M204 and Klipper SET_VELOCITY_LIMIT
   commands that might be emitted by your slicer.
6. Configure the per-feature acceleration control values by adding the following
   block to your Start G-Code, before your `PRINT_START` macro. Alternatively you 
   can also use a [config file](./config/example.conf) by adding ` -c /path/to/config/file` 
   to the call in the post-processing options.  
   ```text
   ; ACCEL: 10000/10000/20  for TYPE:Travel
   ; ACCEL: 2000/1000/5     for TYPE:First Layer
   ; ACCEL: 2000/1000/5     for TYPE:Custom
   ; ACCEL: 2000/1000/5     for TYPE:External perimeter
   ; ACCEL: 2000/1000/5     for TYPE:Overhang perimeter
   ; ACCEL: 4000/2000/10    for TYPE:Internal perimeter
   ; ACCEL: 2000/1000/5     for TYPE:Top solid infill
   ; ACCEL: 10000/5000/10   for TYPE:Solid infill
   ; ACCEL: 10000/5000/20   for TYPE:Internal infill
   ; ACCEL: 5000/2500/5     for TYPE:Bridge infill
   ; ACCEL: 5000/2500/5     for TYPE:Internal bridge infill
   ; ACCEL: 2000/1000/5     for TYPE:Thin wall
   ; ACCEL: 2000/1000/5     for TYPE:Gap fill
   ; ACCEL: 5000/2500/5     for TYPE:Skirt
   ; ACCEL: 10000/5000/20   for TYPE:Support material
   ; ACCEL: 5000/2500/5     for TYPE:Support material interface
   ```

   Accelerations are specified in the ACCEL / ACCEL_TO_DECEL / SQUARE_CORNER_VELOCITY format.

## How does it work

Slic3r-based Slicers prefix blocks of print moves with `;TYPE:External
perimeter` style comments. These comments are used by this post-processor to
pick the right values from the configuration block in the start G-Code. All
acceleration values are used until a different type of comment is detected.
Travel moves are automatically detected and use the `TYPE:Travel` setting. After
travel is done this post-processor goes back to using the current per-feature
accelerations.
