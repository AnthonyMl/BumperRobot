Bumper Robot

A WIP bot for the Ys video game.
The application is Windows specific and has some configuration dependent constants.
Designed only to handle the second last boss for ease of detection as
- the background on bosses is static
- this boss' movement and attack patterns are independent of input

TODOs:
- detect enemy attacks
- avoid enemy attacks
- predict and filter detected objects' state
- Use a method that captures window output rather than screen output as screen input causes feedback and instability when using the overlay. This also causes reproducibility differences between running the overlay and not.
- Use types for more safety when handling rectangles in different coordinate systems
- decouple frame fetching and processing
- wgpu compute for processing the frames
