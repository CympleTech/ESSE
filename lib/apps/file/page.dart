import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/file_image.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

class FolderList extends StatefulWidget {
  @override
  _FolderListState createState() => _FolderListState();
}

const List FILE_DIRECTORY = [
    ["Recent", Icons.label_rounded],
    ["Starred", Icons.star_rounded],
    ["Home", Icons.home_rounded],
    ["Documents", Icons.my_library_books_rounded],
    ["Pictures", Icons.collections_rounded],
    ["Music", Icons.my_library_music_rounded],
    ["Videos", Icons.video_collection_rounded],
    ["Trash", Icons.auto_delete_rounded],
  ];

class _FolderListState extends State<FolderList> {
  int chooseIndex = 0;

  @override
  void initState() {
    super.initState();
    Future.delayed(Duration.zero, () {
        final isDesktop = isDisplayDesktop(context);
        if (isDesktop) {
          loadFolder(true, chooseIndex);
        }
    });
  }

  loadFolder(bool isDesktop, int index) async {
    final widget = FilePage(title: FILE_DIRECTORY[index][0]);
    if (isDesktop) {
      Provider.of<AccountProvider>(context, listen: false).updateActivedApp(widget);
    } else {
      Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
    }
  }

  changeItem(int index, bool isDesktop) {
    setState(() {
        chooseIndex = index;
        loadFolder(isDesktop, index);
    });
  }

  Widget item(int index, ColorScheme color, bool isDesktop) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () => changeItem(index, isDesktop),
      child: SizedBox(
        height: 55.0,
        child: Row(
          children: [
            Container(
              width: 45.0,
              height: 45.0,
              margin: const EdgeInsets.only(left: 20.0, right: 15.0),
              child: Icon(FILE_DIRECTORY[index][1], size: 24.0, color: color.primary),
              decoration: BoxDecoration(
                color: color.surface,
                borderRadius: BorderRadius.circular(15.0)
              ),
            ),
            chooseIndex == index
            ? Text(FILE_DIRECTORY[index][0], style: TextStyle(fontSize: 16.0, color: color.primary))
            : Text(FILE_DIRECTORY[index][0], style: TextStyle(fontSize: 16.0))
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final isDesktop = isDisplayDesktop(context);

    return Expanded(
      child: ListView.builder(
        itemCount: FILE_DIRECTORY.length,
        itemBuilder: (BuildContext ctx, int index) => item(index, color, isDesktop),
    ));
  }
}

class FilePage extends StatelessWidget {
  final String title;
  const FilePage({Key key, this.title}) : super(key: key);

  String remove_dir(String name) {
    if (name.endsWith('.dir')) {
      final i = name.lastIndexOf('.');
      return name.substring(0, i);
    }

    return name;
  }

  Widget item(String name) {
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
            padding: const EdgeInsets.only(bottom: 10.0),
            child: fileIcon(name, 48.0),
          ),
          Tooltip(
            message: trueName,
            child: Text(trueName, style: TextStyle(fontSize: 16.0), maxLines: 1, overflow: TextOverflow.ellipsis),
          )
        ]
      )
    );
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Scaffold(
      body: SafeArea(
        child: Padding(
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
                    item('myworks.dir'),
                    item('ESSE-infos-public.dir'),
                    item('personal.dir'),
                    item('others.dir'),
                    item('logo.jpg'),
                    item('cat.png'),
                    item('what-is-esse_en.doc'),
                    item('20210101-customers.xls'),
                    item('product.pdf'),
                    item('deck.ppt'),
                    item('coder.md'),
                    item('how-to-live-in-happy.mp4'),
                    item('something_important'),
                    item('car.json'),
                  ],
                )
              )
            ]
          )
        )
      )
    );
  }
}
