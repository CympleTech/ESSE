import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/provider.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';

import 'package:esse/apps/file/models.dart';
import 'package:esse/apps/file/editor.dart';

class FilesList extends StatefulWidget {
  List<FilePath> path; // current file/folder.
  FilesList({Key? key, required this.path}) : super(key: key);

  @override
  _FilesListState createState() => _FilesListState(this.path.last.root);
}

class _FilesListState extends State<FilesList> {
  RootDirectory root; // check if root is changed.
  bool _isDesktop = false;
  List<FilePath> _children = []; // children if is folder.

  _FilesListState(this.root);

  @override
  void initState() {
    super.initState();
    rpc.addListener('dc-list', _dcList, false);
    rpc.addListener('dc-file-create', _dcFileCreate, false);
    rpc.addListener('dc-folder-create', _dcFolderCreate, false);
    rpc.addListener('dc-file-upload', _dcFolderCreate, false);

    _loadDirectory(widget.path.last);
  }

  _dcList(List params) {
    this._children.clear();
    params.forEach((param) {
        this._children.add(FilePath.fromList(param));
    });
    setState(() {});
  }

  _dcFileCreate(List params) {
    final newFile = FilePath.fromList(params);
    _navigator(EditorPage(path: newFile, parents: widget.path));
  }

  _dcFolderCreate(List params) {
    setState(() {
        this._children.add(FilePath.fromList(params));
    });
  }

  _loadDirectory(FilePath path) {
    rpc.send('dc-list', [path.root.toInt(), path.id]);
  }

  _navigator(Widget w) {
    if (_isDesktop) {
      context.read<AccountProvider>().updateActivedWidget(w);
    } else {
      Navigator.push(context, MaterialPageRoute(builder: (_) => w));
    }
  }

  _prevDirectory(int i) {
    setState(() {
        widget.path = List.generate(i+1, (j) => widget.path[j]);
        _loadDirectory(widget.path.last);
    });
  }


  _nextDirectory(FilePath path) {
    setState(() {
        widget.path.add(path);
        _loadDirectory(path);
    });
  }

  List<Widget> _pathWidget() {
    List<Widget> widgets = [];

    final n = widget.path.length;
    for (int i = 0; i < n; i++) {
      widgets.add(InkWell(
          onTap: () => _prevDirectory(i),
          child: Text('/'+widget.path[i].directoryName(),
            style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB)))
      ));
    }

    return widgets;
  }

  _showItemMenu(details, lang, FilePath file) async {
    final screenSize = MediaQuery.of(context).size;
    double left = details.globalPosition.dx;
    double top = details.globalPosition.dy;
    await showMenu(
      context: context,
      position: RelativeRect.fromLTRB(
        left,
        top,
        screenSize.width - left,
        screenSize.height - top,
      ),
      items: [
        file.starred ? PopupMenuItem<int>(
          value: 0,
          child: Text(lang.setunstar, style: TextStyle(color: Color(0xFF6174FF))),
        ) : PopupMenuItem<int>(
          value: 0,
          child: Text(lang.setstar, style: TextStyle(color: Color(0xFF6174FF))),
        ),
        PopupMenuItem<int>(
          value: 1,
          child: Text(lang.rename, style: TextStyle(color: Color(0xFF6174FF))),
        ),
        PopupMenuItem<int>(
          value: 2,
          child: Text(lang.moveTo, style: TextStyle(color: Color(0xFF6174FF))),
        ),
        PopupMenuItem<int>(
          value: 8,
          child: Text(lang.moveTrash, style: TextStyle(color: Color(0xFF6174FF))),
        ),
        PopupMenuItem<int>(
          value: 9,
          child: Text(lang.deleteImmediate, style: TextStyle(color: Colors.red)),
        ),
      ],
      elevation: 8.0,
    ).then((value) {
        if (value == 0) {
          // star/unstar
          rpc.send('dc-file-star', [file.id, !file.starred]);
          _loadDirectory(widget.path.last);
        } else if (value == 1) {
          // rename
          showShadowDialog(context, Icons.edit_rounded, lang.rename,
            _RenameScreen(file: file), 20.0
          );
        } else if (value == 2) {
          // moveTo
          showShadowDialog(context, Icons.drive_file_move_rounded, lang.moveTo,
            _MoveToScreen(file: file, path: widget.path),
          );
        } else if (value == 8) {
          // trash
          rpc.send('dc-file-trash', [file.id]);
          _loadDirectory(widget.path.last);
        } else if (value == 9) {
          // delete
          showDialog(
            context: context,
            builder: (BuildContext context) {
              return AlertDialog(
                title: Text(lang.delete + " ${file.showName()} ?"),
                actions: [
                  TextButton(child: Text(lang.cancel), onPressed: () => Navigator.pop(context)),
                  TextButton(child: Text(lang.ok),
                    onPressed:  () {
                      Navigator.pop(context);
                      rpc.send('dc-file-delete', [file.id]);
                      _loadDirectory(widget.path.last);
                    },
                  ),
                ]
              );
            },
          );
        }
    });
  }

  Widget _item(FilePath file, AppLocalizations lang, bool desktop, ColorScheme color) {
    final trueName = file.showName();
    final params = file.fileType().params();

    return GestureDetector(
      onLongPressDown: desktop ? null : (details) => _showItemMenu(details, lang, file),
      onSecondaryLongPressDown: desktop ? (details) => _showItemMenu(details, lang, file) : null,
      onTap: () {
        if (file.isDirectory()) {
          _nextDirectory(file);
        } else if (file.isPost()) {
          _navigator(EditorPage(path: file, parents: widget.path));
        }
      },
      child: Container(
        padding: const EdgeInsets.all(4.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Stack(
                alignment: Alignment.center,
                children: [
                  Icon(params[0], color: params[1], size: 48.0),
                  if (file.starred)
                  Positioned(bottom: 2.0, right: 2.0,
                    child: Container(
                      decoration: ShapeDecoration(color: color.background, shape: CircleBorder()),
                      child: Icon(Icons.star_rounded, color: Color(0xFF6174FF), size: 16.0),
                    ),
                  ),
                ]
            ),
            Tooltip(
              message: trueName,
              child: Text(trueName,
                style: TextStyle(fontSize: 14.0), maxLines: 1, overflow: TextOverflow.ellipsis),
            )
          ]
    )));
  }

  @override
  Widget build(BuildContext context) {
    if (widget.path.last.root != this.root) {
      this.root = widget.path.last.root;
      _loadDirectory(widget.path.last);
    }

    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    this._isDesktop = isDisplayDesktop(context);
    final desktopDevice = isDesktop();

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.dataCenter),
        actions: [
          if (widget.path.last.root != RootDirectory.Star)
          widget.path.last.root == RootDirectory.Trash
          ? IconButton(
            icon: Icon(Icons.delete_forever, color: Colors.red),
            onPressed: () => showDialog(
              context: context,
              builder: (BuildContext context) {
                return AlertDialog(
                  title: Text(lang.trashClear),
                  actions: [
                    TextButton(
                      child: Text(lang.cancel),
                      onPressed: () => Navigator.pop(context),
                    ),
                    TextButton(
                      child: Text(lang.ok),
                      onPressed:  () {
                        Navigator.pop(context);
                        //rpc.send('trash-clear', []);
                      },
                    ),
                  ]
                );
              },
            )
          )
          : PopupMenuButton<int>(
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(15)
            ),
            color: const Color(0xFFEDEDED),
            child: SizedBox(width: 40.0, child: Icon(Icons.add_rounded, color: color.primary)),
            onSelected: (int value) async {
              final parent = widget.path.last;
              if (value == 0) {
                rpc.send('dc-file-create',
                  [parent.root.toInt(), parent.id, FilePath.newPostName(lang.newPost)]
                );
              } else if (value == 1) {
                showShadowDialog(context, Icons.folder_rounded, lang.newFolder,
                  _CreateFolder(parent: parent), 20.0
                );
              } else if (value == 2) {
                final file = await pickFile();
                if (file != null) {
                  rpc.send('dc-file-upload', [parent.root.toInt(), parent.id, file]);
                }
              }
            },
            itemBuilder: (context) {
              return <PopupMenuEntry<int>>[
                PopupMenuItem<int>(value: 0,
                  child: Text(lang.newPost, style: TextStyle(color: Colors.black, fontSize: 16.0)),
                ),
                PopupMenuItem<int>(value: 1,
                  child: Text(lang.newFolder, style: TextStyle(color: Colors.black, fontSize: 16.0)),
                ),
                PopupMenuItem<int>(value: 2,
                  child: Text(lang.uploadFile, style: TextStyle(color: Colors.black, fontSize: 16.0)),
                ),
              ];
            }
          ),
          const SizedBox(width: 10.0),
        ]
      ),
      body: Padding(
        padding: const EdgeInsets.all(10.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Row(children: this._pathWidget()),
            const SizedBox(height: 4.0),
            Expanded(
              child: GridView.extent(
                maxCrossAxisExtent: 75.0,
                childAspectRatio: 0.8,
                children: this._children.map((file) => _item(file, lang, desktopDevice, color)).toList()
              ),
            )
          ]
        )
      ),
    );
  }
}

class _CreateFolder extends StatelessWidget {
  final FilePath parent;
  TextEditingController _nameController = TextEditingController();
  FocusNode _nameFocus = FocusNode();

  _CreateFolder({Key? key, required this.parent}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    _nameFocus.requestFocus();

    return Column(
      children: [
        Container(
          padding: EdgeInsets.only(bottom: 20.0),
          child: InputText(
            icon: Icons.folder_rounded,
            text: lang.newFolder,
            controller: _nameController,
            focus: _nameFocus),
        ),
        ButtonText(
          text: lang.send,
          action: () {
            final name = _nameController.text.trim();
            if (name.length == 0) {
              return;
            }
            rpc.send('dc-folder-create',
              [parent.root.toInt(), parent.id, FilePath.newFolderName(name)]
            );
            Navigator.pop(context);
        }),
      ]
    );
  }
}

class _RenameScreen extends StatelessWidget {
  final FilePath file;
  TextEditingController _nameController = TextEditingController();
  FocusNode _nameFocus = FocusNode();

  _RenameScreen({Key? key, required this.file}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    _nameFocus.requestFocus();
    if (_nameController.text.trim().length == 0) {
      _nameController.text = file.showName();
    }

    return Column(
      children: [
        Container(
          padding: EdgeInsets.only(bottom: 20.0),
          child: InputText(
            icon: Icons.folder_rounded,
            text: lang.newFolder,
            controller: _nameController,
            focus: _nameFocus),
        ),
        ButtonText(
          text: lang.send,
          action: () {
            final name = _nameController.text.trim();
            if (name.length == 0) {
              return;
            }
            rpc.send('dc-file-update',
              [file.id, file.root.toInt(), file.parent, file.rename(name)]
            );
            rpc.send('dc-list', [file.root.toInt(), file.parent]);
            Navigator.pop(context);
        }),
      ]
    );
  }
}

class _MoveToScreen extends StatefulWidget {
  final List<FilePath> path;
  final FilePath file;
  _MoveToScreen({Key? key, required this.file, required this.path}) : super(key: key);

  @override
  _MoveToScreenState createState() => _MoveToScreenState(List.from(this.path));
}

class _MoveToScreenState extends State<_MoveToScreen> {
  List<FilePath> path = [];
  List<FilePath> _list = [];
  int? _selected;

  _MoveToScreenState(this.path);

  @override
  void initState() {
    super.initState();
    _loadDirectories();
  }

  _loadDirectories() async {
    final res = await httpPost(Global.httpRpc, 'dc-list',
      [this.path.last.root.toInt(), this.path.last.id]);
    if (res.isOk) {
      this._list.clear();
      this._selected = null;
      res.params.forEach((param) {
          final f = FilePath.fromList(param);
          if (f.isDirectory()) {
            this._list.add(f);
          }
      });
      setState(() {});
    } else {
      // TODO toast.
      print(res.error);
    }
  }

  List<Widget> _pathWidget(AppLocalizations lang) {
    List<Widget> widgets = [];

    if (this.path.length > 0) {
      widgets.add(IconButton(
          icon: Icon(Icons.arrow_upward_rounded, size: 20.0, color: Color(0xFF6174FF)),
          onPressed: () {
            this.path.removeLast();
            if (this.path.length > 0) {
              _loadDirectories();
            } else {
              this._list.clear();
              this._selected = null;
              ROOT_DIRECTORY.forEach((root) {
                  final name = root.params(lang)[1];
                  this._list.add(FilePath.root(root, name));
              });
              setState(() {});
            }
          }
      ));
    }

    final n = this.path.length;
    for (int i = 0; i < n; i++) {
      widgets.add(InkWell(
          onTap: () {
            this.path = List.generate(i+1, (j) => this.path[j]);
            _loadDirectories();
          },
          child: Text('/'+this.path[i].directoryName(),
            style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB)))
      ));
    }

    return widgets;
  }

  Widget _item(FilePath file, int index) {
    return ListTile(
      leading: Icon(Icons.folder_rounded),
      title: InkWell(
        child: Padding(
          padding: const EdgeInsets.all(2.0),
          child: Text(file.showName(), style: TextStyle(fontSize: 15.0)),
        ),
        onTap: () {
          this.path.add(file);
          _loadDirectories();
        }
      ),
      trailing: Radio(
        value: index,
        groupValue: _selected,
        onChanged: (int? n) => setState(() {
            _selected = index;
        }),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    double maxHeight = MediaQuery.of(context).size.height - 400;
    if (maxHeight < 100.0) {
      maxHeight = 100.0;
    }

    return Column(
      children: [
        Row(children: this._pathWidget(lang)),
        Container(
          height: maxHeight,
          child: SingleChildScrollView(
            child: Column(
              children: List<Widget>.generate(_list.length, (i) => _item(_list[i], i),
            ))
          )
        ),
        ButtonText(
          text: lang.ok,
          enable: this._selected != null,
          action: () {
            final parent = this._list[this._selected!];
            rpc.send('dc-file-update',
              [widget.file.id, parent.root.toInt(), parent.id, widget.file.name]
            );
            Navigator.pop(context);
            rpc.send('dc-list', [widget.path.last.root.toInt(), widget.path.last.parent]);
        }),
      ]
    );
  }
}
