# Folder icon maker 📁

Folder icon maker lets you easily add images on top of a folder icon. It is mostly meant for Gnome. 
This application is my first attempt at creating an application using Rust and Libadwaita.

## Todo 📝
These are ideas I want to implement. Everything with a `*` I want implemented before I want to release it
- [ ] Create Icon*
- [ ] Add features
    - [ ] Automatically load the folder Icon*
    - [ ] Support SVG's*
    - [ ] Drag and drop
    - [ ] Proper error handling
    - [ ] Add threading
        - [ ] Loading images*
        - [ ] Saving images*
    - [ ] Start Screen
    - [ ] Export to SVG
    - [ ] Change folder image in preferences
    - [ ] Add warning if closing with unsaved changes*
    - [ ] Convert top image to Greyscale*
        - [ ] Slider for threshhold
        - [ ] Select color* 
    - [ ] Add key shortcuts*
- [ ] Clean up code*
- [ ] Add comments
- [ ] Correct flatpak manifest*
- [ ] Think of better name

## Contributing 🤝
This program is mostly meant as practise for me so I want to solve most problems by myself. So the best will be to create an issue if you encounter any.
If you want to create a merge request. That is off course totally fine, but please try not to fundamentally change how it works and clearly explain what you did and how it works 😁

## Running the app 🏃
The app is currently not finished, so I have not released it anywhere yet. I do however plan to release it on flathub.
The easiest way to run the app is:
1. Clone the repo
2. Open it in [gnome-builder](https://flathub.org/apps/org.gnome.Builder)
3. Start the application by pressing `ctrl+shift+escape`

## Credits 🫂
Wow documentation is really hard to understand so I used few programs as inspiration and to learn how everything works, so massive shout-out to:
- Eyedropper - https://github.com/FineFindus/eyedropper
- Switcheroo - https://gitlab.com/adhami3310/Switcheroo
- Geopard - https://github.com/ranfdev/Geopard
