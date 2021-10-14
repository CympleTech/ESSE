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

  @override
  initState() {
    print("File editor initState...");
    super.initState();
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
            final w = FilesList(path: widget.path);
            context.read<AccountProvider>().updateActivedWidget(w);
          }
        ) : null,
        centerTitle: true,
        title: TextButton(child: Text('New Document 0'),
          onPressed: () {}
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
