# Folder icon maker 📁

Folder icon maker lets you easily add images on top of a folder icon. It is mostly meant for Gnome. 
This application is my first attempt at creating an application using Rust and Libadwaita.
![afbeelding](https://github.com/youpie/Folder_icon_creator/assets/37704067/c20537a0-39ca-486c-87a4-f994f43c3cc6)


## Todo 📝
These are ideas I want to implement. Everything with a `*` I want implemented before I want to release it
- [ ] Create Icon*
- [ ] Add features
    - [X] Automatically load the folder Icon*
    - [X] Support SVG's*
    - [ ] Drag and drop
    - [ ] Proper error handling
    - [X] Add threading
        - [X] Loading images*
        - [X] Saving images*
    - [ ] Start Screen
    - [ ] Export to SVG
    - [ ] Change folder image in preferences
    - [X] Add warning if closing with unsaved changes*
    - [ ] Convert top image to Greyscale*
        - [ ] Slider for threshhold
        - [ ] Select color* 
    - [ ] Add key shortcuts*
        - [ ] Save
        - [ ] Open 
    - [ ] Rounded corner option for top image
    - [X] Load image folder on start*
    - [X] Ability to save and load settings
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
