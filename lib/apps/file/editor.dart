import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:flutter_quill/flutter_quill.dart' hide Text;

import 'package:esse/utils/adaptive.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/file/models.dart';
import 'package:esse/apps/file/list.dart';

class EditorPage extends StatefulWidget {
  final FilePath path;
  const EditorPage({Key? key, required this.path}) : super(key: key);

  @override
  _EditorPageState createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  QuillController _controller = QuillController.basic();
  TextEditingController _nameController = TextEditingController();
  bool _nameEdit = false;

  @override
  initState() {
    print("File editor initState...");
    super.initState();
    _nameController.text = widget.path.name();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    return Scaffold(
      appBar: AppBar(
        leading: isDesktop ? IconButton(
          icon: Icon(Icons.arrow_back),
          onPressed: () {
            final w = FilesList(path: FilePath.prev(widget.path));
            context.read<AccountProvider>().updateActivedWidget(w);
          }
        ) : null,
        centerTitle: true,
        title: _nameEdit
        ? Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 200.0,
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
              onTap: () {
                if (_nameController.text.length > 0) {
                  // TODO update file name.
                }
                setState(() {
                    this._nameEdit = false;
                });
              },
              child: Container(
                width: 20.0,
                child: Icon(Icons.done_rounded, color: color.primary)),
            ),
            const SizedBox(width: 8.0),
            GestureDetector(
              onTap: () => setState(() {
                  _nameController.text = widget.path.name();
                  this._nameEdit = false;
              }),
              child: Container(
                width: 20.0, child: Icon(Icons.clear_rounded)),
            ),
        ])
        : TextButton(child: Text(widget.path.name()),
          onPressed: () => setState(() { this._nameEdit = true; }),
        ),
      ),
      body: Column(
        children: [
          Container(
            width: double.infinity,
            decoration: BoxDecoration(color: color.secondary),
            padding: const EdgeInsets.all(10.0),
            child: QuillToolbar.basic(
              controller: _controller,
              showAlignmentButtons: false,
            )
          ),
          Expanded(
            child: Container(
              padding: const EdgeInsets.all(10.0),
              child: QuillEditor.basic(
                controller: _controller,
                readOnly: false, // true for view only mode
              ),
            ),
          )
        ],
      )
    );
  }
}
