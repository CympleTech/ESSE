import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/file_image.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

class FilesList extends StatefulWidget {
  @override
  _FilesListState createState() => _FilesListState();
}

const List FILE_DIRECTORY = [
    ["Starred", Icons.star],
    ["Documents", Icons.description],
    ["Pictures", Icons.photo],
    ["Media", Icons.music_video],
    ["Trash", Icons.auto_delete],
  ];

class _FilesListState extends State<FilesList> {
  @override
  void initState() {
    super.initState();
    loadRecents();
  }

  loadRecents() {
    //
  }

  changeItem(String name, bool isDesktop) {
    setState(() {
        // chooseIndex = index;
        // loadFolder(isDesktop, index);
    });
  }

  Widget item(String name, IconData icon, ColorScheme color, bool isDesktop) {
    return Container(
      width: 140,
      height: 50,
      decoration: BoxDecoration(
        color: color.surface,
        borderRadius: BorderRadius.circular(15.0),
      ),
      child: InkWell(
        onTap: () => changeItem(name, isDesktop),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Padding(
              padding: const EdgeInsets.only(left: 16.0, right: 8.0),
              child: Icon(icon, color: color.primary),
            ),
            Expanded(
              child: Text(name, style: TextStyle(fontSize: 14.0))
            )
          ]
      )),
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    final List<Widget> widgets = FILE_DIRECTORY.map(
      (i) => item(i[0], i[1], color, isDesktop)
    ).toList();

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.files + ' (${lang.wip})'),
        bottom: PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)
        ),
      ),
      body: Padding(
        padding: const EdgeInsets.all(10.0),
        child: ListView(
          children: [
            const SizedBox(height: 10.0),
            Wrap(
              spacing: 8.0,
              runSpacing: 8.0,
              alignment: WrapAlignment.start,
              children: widgets
            ),
            const SizedBox(height: 36.0),
            Text('Recents', style: Theme.of(context).textTheme.title),
            const SizedBox(height: 16.0),
            Wrap(
              spacing: 4.0,
              runSpacing: 8.0,
              alignment: WrapAlignment.start,
              children: <Widget> [
                FileItem(name: 'myworks.dir'),
                FileItem(name: 'ESSE-infos-public.dir'),
                FileItem(name: 'personal.dir'),
                FileItem(name: 'others.dir'),
                FileItem(name: 'logo.jpg'),
                FileItem(name: 'cat.png'),
                FileItem(name: 'what-is-esse_en.doc'),
                FileItem(name: '20210101-customers.xls'),
                FileItem(name: 'product.pdf'),
                FileItem(name: 'deck.ppt'),
                FileItem(name: 'coder.md'),
                FileItem(name: 'how-to-live-in-happy.mp4'),
                FileItem(name: 'something_important'),
                FileItem(name: 'car.json'),
              ],
            ),
          ]
        )
      ),
    );
  }
}

class FileItem extends StatelessWidget {
  final String name;
  const FileItem({Key key, this.name}) : super(key: key);

  String remove_dir(String name) {
    if (name.endsWith('.dir')) {
      final i = name.lastIndexOf('.');
      return name.substring(0, i);
    }
    return name;
  }

  @override
  Widget build(BuildContext context) {
    final trueName = remove_dir(name);
    return Container(
      width: 80.0,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Container(
            height: 60.0,
            width: 60.0,
            child: fileIcon(name, 48.0),
          ),
          Tooltip(
            message: trueName,
            child: Text(trueName,
              style: TextStyle(fontSize: 14.0), maxLines: 1, overflow: TextOverflow.ellipsis),
          )
        ]
      )
    );
  }
}

class FilePage extends StatelessWidget {
  final String title;
  const FilePage({Key key, this.title}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Scaffold(
      body: Padding(
        padding: const EdgeInsets.all(10.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: isDesktop ? CrossAxisAlignment.start : CrossAxisAlignment.center,
          children: <Widget>[
            Row(
              children: [
                if (!isDesktop)
                GestureDetector(
                  onTap: () => Navigator.pop(context),
                  child: Container(width: 20.0, child: Icon(Icons.arrow_back, color: color.primary)),
                ),
                const SizedBox(width: 15.0),
                Expanded(child: Text(title, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 20.0))),
                PopupMenuButton<int>(
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(15)
                  ),
                  color: const Color(0xFFEDEDED),
                  child: Icon(Icons.add_rounded, color: color.primary),
                  onSelected: (int value) {
                    if (value == 0) {
                      // new post
                    } else if (value == 1) {
                      // new folder
                    } else if (value == 2) {
                      // upload file
                    }
                  },
                  itemBuilder: (context) {
                    return <PopupMenuEntry<int>>[
                      PopupMenuItem<int>(value: 0,
                        child: Text('New Post', style: TextStyle(color: Colors.black, fontSize: 16.0)),
                      ),
                      PopupMenuItem<int>(value: 1,
                        child: Text('New Folder', style: TextStyle(color: Colors.black, fontSize: 16.0)),
                      ),
                      PopupMenuItem<int>(value: 2,
                        child: Text('Upload File', style: TextStyle(color: Colors.black, fontSize: 16.0)),
                      ),
                    ];
                  }
                ),
                const SizedBox(width: 15.0),
                GestureDetector(
                  onTap: () {}, // view_module_rounded
                  child: Container(width: 20.0, child: Icon(Icons.view_list_rounded, color: color.primary)),
                ),
                const SizedBox(width: 10.0),
              ],
            ),
            SizedBox(height: 5.0),
            Row(
              children: [
                const SizedBox(width: 15.0),
                InkWell(
                  onTap: () {
                    print('Home');
                  },
                  child: Text('Home', style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB)))
                ),
                Text('/', style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB))),
                InkWell(
                  onTap: () {
                    print('Home/workspace');
                  },
                  child: Text('workspace', style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB)))
                ),
                Text('/', style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB))),
                InkWell(
                  onTap: () {
                    print('Home/workspace/cymple');
                  },
                  child: Text('cymple', style: TextStyle(fontSize: 14.0, color: Color(0xFFADB0BB)))
                ),
              ]
            ),
            SizedBox(height: 15.0),
            Expanded(
              child: Wrap(
                spacing: 4.0,
                runSpacing: 16.0,
                alignment: WrapAlignment.start,
                children: <Widget> [
                  FileItem(name: 'myworks.dir'),
                  FileItem(name: 'ESSE-infos-public.dir'),
                  FileItem(name: 'personal.dir'),
                  FileItem(name: 'others.dir'),
                  FileItem(name: 'logo.jpg'),
                  FileItem(name: 'cat.png'),
                  FileItem(name: 'what-is-esse_en.doc'),
                  FileItem(name: '20210101-customers.xls'),
                  FileItem(name: 'product.pdf'),
                  FileItem(name: 'deck.ppt'),
                  FileItem(name: 'coder.md'),
                  FileItem(name: 'how-to-live-in-happy.mp4'),
                  FileItem(name: 'something_important'),
                  FileItem(name: 'car.json'),
                ],
              )
            )
          ]
        )
      )
    );
  }
}
