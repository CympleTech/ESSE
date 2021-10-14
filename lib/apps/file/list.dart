import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/file_image.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/file/models.dart';
import 'package:esse/apps/file/editor.dart';

class FilesList extends StatefulWidget {
  FilePath path;
  FilesList({Key? key, required this.path}) : super(key: key);

  @override
  _FilesListState createState() => _FilesListState();
}

class _FilesListState extends State<FilesList> {
  @override
  void initState() {
    super.initState();
    _loadDirectory();
  }

  _loadDirectory() {
    //
  }

  _nextDirectory(FilePath path) {
    setState(() {
        widget.path = path;
        _loadDirectory();
    });
  }

  List<Widget> _pathWidget(String root) {
    List<Widget> widgets = [
      InkWell(
        onTap: () => _nextDirectory(FilePath.root(widget.path.root)),
        child: Text(root,
          style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB)))
      )
    ];

    final n = widget.path.path.length;
    for (int i = 0; i < n; i++) {
      final name = widget.path.path[i];
      final current_path = List.generate(i+1, (i) => widget.path.path[i]);
      widgets.add(InkWell(
          onTap: () => _nextDirectory(FilePath(widget.path.root, current_path)),
          child: Text('/'+FilePath.directoryName(name),
            style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB)))
      ));
    }

    return widgets;
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.dataCenter + ' (${lang.wip})'),
        actions: [
          widget.path.root == RootDirectory.Trash
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
            onSelected: (int value) {
              if (value == 0) {
                final editor = EditorPage(path: widget.path);
                if (!isDesktop) {
                  Navigator.push(context, MaterialPageRoute(builder: (_) => editor));
                } else {
                  context.read<AccountProvider>().updateActivedWidget(editor);
                }
              } else if (value == 1) {
                // new folder
              } else if (value == 2) {
                // upload file
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
            Row(children: this._pathWidget('/' + widget.path.root.params(lang)[1])),
            const SizedBox(height: 4.0),
            Expanded(
              child: GridView.extent(
                maxCrossAxisExtent: 75.0,
                childAspectRatio: 0.8,
                children: <Widget> [
                  FileItem(
                    path: FilePath.next(widget.path, 'myworks.dir'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'ESSE-infos-public.dir'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'personal.dir'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'others.dir'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'logo.jpg'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'cat.png'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'what-is-esse_en.doc'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, '20210101-customers.xls'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'product.pdf'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'deck.ppt'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'coder.md'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'how-to-live-in-happy.mp4'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'something_important'),
                    directory: this._nextDirectory),
                  FileItem(path: FilePath.next(widget.path, 'car.json'),
                    directory: this._nextDirectory),
                ],
              ),
            )
          ]
        )
      ),
    );
  }
}

class FileItem extends StatelessWidget {
  final FilePath path;
  final Function directory;
  const FileItem({Key? key, required this.path, required this.directory}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final trueName = this.path.name();
    return InkWell(
      onTap: () {
        if (this.path.isDirectory()) {
          this.directory(this.path);
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
              child: fileIcon(this.path.fullName, 48.0),
            ),
            Tooltip(
              message: trueName,
              child: Text(trueName,
                style: TextStyle(fontSize: 14.0), maxLines: 1, overflow: TextOverflow.ellipsis),
            )
          ]
    )));
  }
}
