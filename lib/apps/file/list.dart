import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/provider.dart';
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

  Widget _item(FilePath file) {
    final trueName = file.showName();
    final params = file.fileType().params();
    return InkWell(
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
            Container(
              height: 55.0,
              width: 60.0,
              child: Icon(params[0], color: params[1], size: 48.0),
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
                children: this._children.map((file) => _item(file)).toList()
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
