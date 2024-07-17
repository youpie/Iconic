
subdir('icons')
# Desktop File
desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = i18n.merge_file(
  type: 'desktop',
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(base_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'applications'
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

# Appdata
appdata_conf = configuration_data()
appdata_conf.set('app-id', base_id)
appdata_conf.set('gettext-package', gettext_package)
appdata_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.metainfo.xml.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: appdata_conf
  ),
  output: '@0@.metainfo.xml'.format(base_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'metainfo'
)

# GSchema
gschema_conf = configuration_data()
gschema_conf.set('app-id', application_id)
configure_file(
  input: '@0@.gschema.xml.in'.format(base_id),
  output: '@0@.gschema.xml'.format(application_id),
  configuration: gschema_conf,
  install: true,
  install_dir: datadir / 'glib-2.0' / 'schemas'
)

appstreamcli = find_program('appstreamcli', required: false, disabler: true)
test('Validate appstream file', appstreamcli,
     args: ['validate', '--no-net', '--explain', appdata_file])

# install_data('nl.emphisia.icon.gschema.xml',
#   install_dir: get_option('datadir') / 'glib-2.0' / 'schemas'
# )

compile_schemas = find_program('glib-compile-schemas', required: false, disabler: true)
test('Validate schema file',
     compile_schemas,
     args: ['--strict', '--dry-run', meson.current_source_dir()])
