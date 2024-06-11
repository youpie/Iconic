project('folder_icon', 'rust',
          version: '0.1.0',
    meson_version: '>= 0.62.0',
  default_options: [ 'warning_level=2', 'werror=false', ],
)

i18n = import('i18n')
gnome = import('gnome')


base_id = 'nl.emphisia.icon'
prefix = get_option('prefix')
datadir = prefix / get_option('datadir')
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
pkgdatadir = datadir / meson.project_name()

if get_option('profile') == 'development'
  profile = 'Devel'
  application_id = '@0@.@1@'.format(base_id, profile)
else
  profile = ''
  version_suffix = ''
  application_id = base_id
endif

subdir('data')
subdir('src')
subdir('po')

gnome.post_install(
     glib_compile_schemas: true,
    gtk_update_icon_cache: true,
  update_desktop_database: true,
)
