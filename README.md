![logo](https://raw.githubusercontent.com/youpie/Folder_icon_creator/main/data/icons/nl.emphisia.icon.svg)

# Iconic 📁

Iconic lets you easily add images on top of a folder icon. It is mostly meant for Gnome. 
This application is my first attempt at creating an application using Rust and Libadwaita.

<a href='https://flathub.org/apps/details/nl.emphisia.icon'><img width='240' alt='Get it on Flathub' src='https://flathub.org/api/badge?locale=en&light'/></a>

![afbeelding](https://github.com/youpie/Iconic/blob/main/data/screenshots/Main%20screen%20dark.png?raw=true)

## Todo 📝
These are ideas I want to implement.
- [ ] Add features
    - [X] Add threading
        - [ ] Loading dragged images
        - [ ] Making images monochrome
            - Monochrome images get recreated every time the preview is updated. Slows down quite a lot on slower systems, i could also
            - [ ] Just not recreate monochrome images every preview update
    - [ ] Export to SVG
        - This would probably require a full rewrite of the image generation system. Using cairo or something, but I originally didn't use cairo as I did not understand it, and good examples were really scarse 
    - [ ] Rounded corner option for top image
    - [ ] Add guide grid or something
    - [X] Multiple folder colors
        - [x] according to gnome 47 accent color?
            - [X] SVG manipulation?
            - [X] Or just packing the images in with iconic
        - [x] Option in settings
    - [x] Regenerate all (d&d'ed) images automatically
        - This way its possible to regenerate all images, when for example the accent color changes (I assume if the image that nautulus has set as folder image is changed but filename stays the same, it will change in nautulis)
        - [x] Save dragged images with metadata about top layer
        - [x] Store all used top layer images in cache
            - This will use a lot of cache size, so
            - [X] Allow users to turn it off (opt-in/-out on first launch?)
            - [ ] Inform users 
            - [X] Do not store too large images
        - [X] Detect current accent color
    - [ ] Create guide when first starting iconic, to show users dragging and dropping images is possible
    - [ ] Refer users to icon library, so they know where to find many good icons   
- [X] Update main app to gnome 47 runtime
- [ ] Add credits for applications I used to learn in the "about iconic"
- [ ] Move donation link to About iconic (issue #8)
- [ ] Make donation link just point to a random charity
- [ ] Improve accessibility
- [ ] Clean up code
    - I feel like a lot of code is not up to "standards" like rust conventions and stuff, So I need to do some research into that. 
- [ ] Add comments
    - I struggled a lot learning it all, So adding comments, especially at points I got stuck, might help other devs also wanting to learn 
- [ ] Reduce file size
    - I the app is about 17 mb's, way to much for such a simple app imo. I might use a lot of large libs (Like cairo) only for a single purpose, so possibly that could be improved.

## Contributing 🤝
This program is mostly meant as practise for me so I want to solve most problems by myself. So the best will be to create an issue if you encounter any.
If you want to create a merge request. That is off course totally fine, but please try not to fundamentally change how it works and clearly explain what you did and how it works 😁

## Running the app 🏃
If you want to run the app:
1. Clone the repo
2. Open it in [gnome-builder](https://flathub.org/apps/org.gnome.Builder)
3. Start the application by pressing `ctrl+shift+escape`

## Credits 🫂
Wow documentation is really hard to understand so I used few programs as inspiration and to learn how everything works, so massive shout-out to:
- Eyedropper - https://github.com/FineFindus/eyedropper
- Switcheroo - https://gitlab.com/adhami3310/Switcheroo
- Geopard - https://github.com/ranfdev/Geopard
- Obfuscate - https://gitlab.gnome.org/World/obfuscate
- Loupe - https://gitlab.gnome.org/GNOME/loupe

### Icon credits
The icon is just a few already existing icons added together, the following are used:
- Gnome text editor - https://gitlab.gnome.org/GNOME/gnome-text-editor
- Adwaita icons - https://gitlab.gnome.org/GNOME/adwaita-icon-theme

### Folder credits
The folders/colors used in Iconic (Except the pink one as I like my own one better) are taken from 
- Adwaita-Colors - https://github.com/dpejoh/Adwaita-colors/tree/main
