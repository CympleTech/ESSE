import 'dart:io' show File, Directory;
import 'dart:convert' show jsonDecode, jsonEncode;
import 'package:path/path.dart' show basename;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:fluttertoast/fluttertoast.dart';
import 'package:flutter_quill/flutter_quill.dart' hide Text;

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';

import 'package:esse/apps/file/models.dart';
import 'package:esse/apps/file/list.dart';

class EditorPage extends StatefulWidget {
  FilePath path;
  final List<FilePath> parents;
  EditorPage({Key? key, required this.path, required this.parents}) : super(key: key);

  @override
  _EditorPageState createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  QuillController? _controller;
  FToast? _fToast;
  TextEditingController _nameController = TextEditingController();
  bool _nameEdit = false;
  String _filePath = '';
  bool _edit = false;

  readFile() async {
    try {
      final s = await File(this._filePath).readAsString();
      final doc = Document.fromJson(jsonDecode(s));
      setState(() {
          this._controller = QuillController(
            document: doc, selection: const TextSelection.collapsed(offset: 0)
          );
      });
    } catch (e) {
      await File(this._filePath).create(recursive: true);
      final doc = Document()..insert(0, '');
      setState(() {
          this._controller = QuillController(
            document: doc, selection: const TextSelection.collapsed(offset: 0));
      });
    }
  }

  save(String saveOk) async {
    final j = this._controller!.document.toDelta().toJson();
    final s = jsonEncode(j);
    await File(this._filePath).writeAsString(s);
    this._fToast!.showToast(
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 24.0, vertical: 10.0),
        decoration: BoxDecoration(borderRadius: BorderRadius.circular(25.0),
          color: Color(0x2F008000)),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [Icon(Icons.check, color: Colors.green),
            const SizedBox(width: 12.0), Text(saveOk, style: TextStyle(color: Colors.green))],
      )),
      gravity: ToastGravity.BOTTOM,
      toastDuration: Duration(seconds: 2),
    );
  }

  @override
  initState() {
    super.initState();
    _nameController.text = widget.path.showName();
    this._filePath = Global.filePath + widget.path.did;
    readFile();

    this._fToast = FToast();
    this._fToast!.init(context);
  }

  @override
  Widget build(BuildContext context) {
    final lang = AppLocalizations.of(context);
    if (_controller == null) {
      return Scaffold(body: Center(child: Text(lang.waiting)));
    }

    final color = Theme.of(context).colorScheme;
    final isDesktop = isDisplayDesktop(context);

    return Scaffold(
      appBar: AppBar(
        leading: isDesktop ? IconButton(
          icon: Icon(Icons.arrow_back),
          onPressed: () {
            final w = FilesList(path: widget.parents);
            context.read<AccountProvider>().updateActivedWidget(w);
          }
        ) : null,
        centerTitle: true,
        title: _nameEdit
        ? Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 120.0,
              child: TextField(
                autofocus: true,
                style: TextStyle(fontSize: 16.0),
                textAlign: TextAlign.center,
                controller: _nameController,
                decoration: InputDecoration(
                  hintStyle: TextStyle(
                    color: Color(0xFF1C1939).withOpacity(0.25)),
                  filled: false,
                  isDense: true,
                ),
              ),
            ),
            const SizedBox(width: 10.0),
            GestureDetector(
              onTap: () async {
                final name = _nameController.text.trim();
                if (name.length > 0) {
                  final res = await httpPost('dc-file-update', [widget.path.id,
                      widget.path.root.toInt(), widget.path.parent, FilePath.newPostName(name)
                    ]
                  );
                  if (res.isOk) {
                    widget.path = FilePath.fromList(res.params);
                    setState(() {
                        this._nameEdit = false;
                    });
                  } else {
                    print('change name error');
                  }
                }
              },
              child: Container(
                width: 20.0,
                child: Icon(Icons.done_rounded, color: color.primary)),
            ),
            const SizedBox(width: 8.0),
            GestureDetector(
              onTap: () => setState(() {
                  _nameController.text = widget.path.showName();
                  this._nameEdit = false;
              }),
              child: Container(
                width: 20.0, child: Icon(Icons.clear_rounded)),
            ),
        ])
        : TextButton(child: Text(widget.path.showName()),
          onPressed: this._edit ? () => setState(() { this._nameEdit = true; }) : null,
        ),
        actions: [
          const SizedBox(width: 10.0),
          if (this._edit)
          IconButton(icon: Icon(Icons.save_rounded), onPressed: () => save(lang.saveOk)),
          if (this._edit)
          IconButton(icon: Icon(Icons.menu_book_rounded, color: Colors.green),
            onPressed: () async {
              await save(lang.saveOk);
              setState(() { this._edit = false; });
          }),
          if (!this._edit)
          IconButton(icon: Icon(Icons.edit_rounded, color: Colors.green), onPressed: () {
              setState(() { this._edit = true; });
          }),
          const SizedBox(width: 10.0),
        ]
      ),
      body: Column(
        children: [
          if (this._edit)
          Container(
            width: double.infinity,
            decoration: BoxDecoration(color: color.secondary),
            padding: const EdgeInsets.only(left: 10.0, right: 10.0, bottom: 5.0),
            child: QuillToolbar.basic(
              controller: this._controller!,
              showAlignmentButtons: true,
              multiRowsDisplay: isDesktop,
              //onImagePickCallback: _onImagePickCallback,
              //onVideoPickCallback: _onImagePickCallback,
              //mediaPickSettingSelector: _selectMediaPickSetting,
              //filePickImpl: pickMedia,
              showLink: false,
            )
          ),
          Expanded(
            child: Container(
              padding: const EdgeInsets.all(10.0),
              child: QuillEditor.basic(
                controller: this._controller!,
                readOnly: !this._edit,
              ),
            ),
          )
        ],
      )
    );
  }

  // Future<MediaPickSetting?> _selectMediaPickSetting(BuildContext context) {
  //   final lang = AppLocalizations.of(context);
  //   return showDialog<MediaPickSetting>(
  //     context: context,
  //     builder: (ctx) => AlertDialog(
  //       contentPadding: const EdgeInsets.all(20.0),
  //       content: Column(
  //         mainAxisSize: MainAxisSize.min,
  //         children: [
  //           TextButton.icon(
  //             icon: const Icon(Icons.collections),
  //             label: Text(lang.gallery),
  //             onPressed: () => Navigator.pop(ctx, MediaPickSetting.Gallery),
  //           ),
  //           const Divider(height: 32.0),
  //           TextButton.icon(
  //             icon: const Icon(Icons.link),
  //             label: Text(lang.link),
  //             onPressed: () => Navigator.pop(ctx, MediaPickSetting.Link),
  //           )
  //         ],
  //       ),
  //     ),
  //   );
  // }

  Future<String> _checkName(String path, String name) async {
    String tryName = name;
    for( var i=0; i >= 0; i++) {
      if (await File(path + tryName).exists()) {
        tryName = "${i}_" + name;
      } else {
        return path + tryName;
      }
    }

    return path + name;
  }

  // Future<String> _onImagePickCallback(File file) async {
  //   final dir = Directory(Global.filePath + widget.path.did + '_assets');
  //   final isExists = await dir.exists();
  //   if (!isExists) {
  //     await dir.create(recursive: true);
  //   }
  //   final pathname = await _checkName(dir.path + '/', basename(file.path));
  //   final copiedFile = await file.copy(pathname);
  //   return copiedFile.path.toString();
  // }
}
