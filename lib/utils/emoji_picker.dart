import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import './emoji_lists.dart' as emojiList;

/// All the possible categories that [Emoji] can be put into
///
/// All [Category] are shown in the keyboard bottombar
enum Category {
  SMILEYS,
  ANIMALS,
  FOODS,
  TRAVEL,
  ACTIVITIES,
  OBJECTS,
  SYMBOLS,
  FLAGS
}

/// Callback function for when emoji is selected
///
/// The function returns the selected [Emoji] as well as the [Category] from which it originated
typedef void OnEmojiSelected(Emoji emoji, Category category);

/// The Emoji Keyboard widget
///
/// This widget displays a grid of [Emoji] sorted by [Category] which the user can horizontally scroll through.
///
/// There is also a bottombar which displays all the possible [Category] and allow the user to quickly switch to that [Category]
class EmojiPicker extends StatefulWidget {
  @override
  _EmojiPickerState createState() => new _EmojiPickerState();

  /// Number of columns in keyboard grid
  int columns;

  /// Number of rows in keyboard grid
  int rows;

  double maxWidth;

  /// The currently selected [Category]
  ///
  /// This [Category] will have its button in the bottombar darkened
  Category selectedCategory;

  /// The function called when the emoji is selected
  OnEmojiSelected onEmojiSelected;

  /// The background color of the keyboard
  Color bgColor;

  /// The color of the keyboard page indicator
  Color indicatorColor;

  Color progressIndicatorColor;

  Color _defaultBgColor = Color.fromRGBO(242, 242, 242, 1);

  /// Determines the icon to display for each [Category]
  CategoryIcons categoryIcons;

  EmojiPicker({
      Key key,
      @required this.onEmojiSelected,
      @required this.maxWidth,
      this.columns = 7,
      this.rows = 3,
      this.selectedCategory,
      this.bgColor,
      this.indicatorColor = Colors.blue,
      this.progressIndicatorColor = Colors.blue,
      this.categoryIcons,
      //this.unavailableEmojiIcon,
  }) : super(key: key) {
    if (selectedCategory == null) {
      selectedCategory = Category.SMILEYS;
    }

    if (this.bgColor == null) {
      bgColor = _defaultBgColor;
    }

    if (categoryIcons == null) {
      categoryIcons = CategoryIcons();
    }
  }
}

/// Class that defines the icon representing a [Category]
class CategoryIcon {
  /// The icon to represent the category
  IconData icon;

  /// The default color of the icon
  Color color;

  /// The color of the icon once the category is selected
  Color selectedColor;

  CategoryIcon({@required this.icon, this.color, this.selectedColor}) {
    if (this.color == null) {
      this.color = Color.fromRGBO(211, 211, 211, 1);
    }
    if (this.selectedColor == null) {
      this.selectedColor = Color.fromRGBO(178, 178, 178, 1);
    }
  }
}

/// Class used to define all the [CategoryIcon] shown for each [Category]
///
/// This allows the keyboard to be personalized by changing icons shown.
/// If a [CategoryIcon] is set as null or not defined during initialization, the default icons will be used instead
class CategoryIcons {
  /// Icon for [Category.SMILEYS]
  CategoryIcon smileyIcon;

  /// Icon for [Category.ANIMALS]
  CategoryIcon animalIcon;

  /// Icon for [Category.FOODS]
  CategoryIcon foodIcon;

  /// Icon for [Category.TRAVEL]
  CategoryIcon travelIcon;

  /// Icon for [Category.ACTIVITIES]
  CategoryIcon activityIcon;

  /// Icon for [Category.OBJECTS]
  CategoryIcon objectIcon;

  /// Icon for [Category.SYMBOLS]
  CategoryIcon symbolIcon;

  /// Icon for [Category.FLAGS]
  CategoryIcon flagIcon;

  CategoryIcons(
    {this.smileyIcon,
      this.animalIcon,
      this.foodIcon,
      this.travelIcon,
      this.activityIcon,
      this.objectIcon,
      this.symbolIcon,
      this.flagIcon}) {
    if (smileyIcon == null) {
      smileyIcon = CategoryIcon(icon: Icons.tag_faces);
    }
    if (animalIcon == null) {
      animalIcon = CategoryIcon(icon: Icons.pets);
    }
    if (foodIcon == null) {
      foodIcon = CategoryIcon(icon: Icons.fastfood);
    }
    if (travelIcon == null) {
      travelIcon = CategoryIcon(icon: Icons.location_city);
    }
    if (activityIcon == null) {
      activityIcon = CategoryIcon(icon: Icons.directions_run);
    }
    if (objectIcon == null) {
      objectIcon = CategoryIcon(icon: Icons.lightbulb_outline);
    }
    if (symbolIcon == null) {
      symbolIcon = CategoryIcon(icon: Icons.euro_symbol);
    }
    if (flagIcon == null) {
      flagIcon = CategoryIcon(icon: Icons.flag);
    }
  }
}

/// A class to store data for each individual emoji
class Emoji {
  /// The name or description for this emoji
  final String name;

  /// The unicode string for this emoji
  ///
  /// This is the string that should be displayed to view the emoji
  final String emoji;

  Emoji({@required this.name, @required this.emoji});

  @override
  String toString() {
    return "Name: " + name + ", Emoji: " + emoji;
  }
}

class _EmojiPickerState extends State<EmojiPicker> {
  bool loaded = false;
  List<Widget> pages = [];

  int smileyPagesNum;
  int animalPagesNum;
  int foodPagesNum;
  int travelPagesNum;
  int activityPagesNum;
  int objectPagesNum;
  int symbolPagesNum;
  int flagPagesNum;
  List<String> allNames = new List();
  List<String> allEmojis = new List();

  Map<String, String> smileyMap = new Map();
  Map<String, String> animalMap = new Map();
  Map<String, String> foodMap = new Map();
  Map<String, String> travelMap = new Map();
  Map<String, String> activityMap = new Map();
  Map<String, String> objectMap = new Map();
  Map<String, String> symbolMap = new Map();
  Map<String, String> flagMap = new Map();

  @override
  void initState() {
    super.initState();
    if (!loaded) {
      loadEmojis();
    }
  }

  loadEmojis() {
    smileyMap = emojiList.smileys;
    animalMap = emojiList.animals;
    foodMap = emojiList.foods;
    travelMap = emojiList.travel;
    activityMap = emojiList.activities;
    objectMap = emojiList.objects;
    symbolMap = emojiList.symbols;
    flagMap = emojiList.flags;

    allNames.addAll(smileyMap.keys);
    allNames.addAll(animalMap.keys);
    allNames.addAll(foodMap.keys);
    allNames.addAll(travelMap.keys);
    allNames.addAll(activityMap.keys);
    allNames.addAll(objectMap.keys);
    allNames.addAll(symbolMap.keys);
    allNames.addAll(flagMap.keys);

    allEmojis.addAll(smileyMap.values);
    allEmojis.addAll(animalMap.values);
    allEmojis.addAll(foodMap.values);
    allEmojis.addAll(travelMap.values);
    allEmojis.addAll(activityMap.values);
    allEmojis.addAll(objectMap.values);
    allEmojis.addAll(symbolMap.values);
    allEmojis.addAll(flagMap.values);

    smileyPagesNum = (smileyMap.values.toList().length / (widget.rows * widget.columns)).ceil();
    List<Widget> smileyPages = new List();

    for (var i = 0; i < smileyPagesNum; i++) {
      smileyPages.add(Container(
          color: widget.bgColor,
          child: GridView.count(
            shrinkWrap: true,
            primary: true,
            crossAxisCount: widget.columns,
            children: List.generate(widget.rows * widget.columns, (index) {
                if (index + (widget.columns * widget.rows * i) <
                  smileyMap.values.toList().length) {
                  String emojiTxt = smileyMap.values
                  .toList()[index + (widget.columns * widget.rows * i)];

                  return Center(
                    child: FlatButton(
                      padding: EdgeInsets.all(0),
                      child: Center(
                        child: Text(
                          emojiTxt,
                          style: TextStyle(fontSize: 18.0),
                        ),
                      ),
                      onPressed: () {
                        widget.onEmojiSelected(
                          Emoji(
                            name: smileyMap.keys.toList()[
                              index + (widget.columns * widget.rows * i)],
                            emoji: smileyMap.values.toList()[
                              index + (widget.columns * widget.rows * i)]),
                          widget.selectedCategory);
                      },
                  ));
                } else {
                  return Container();
                }
            }),
          ),
      ));
    }

    animalPagesNum = (animalMap.values.toList().length / (widget.rows * widget.columns)).ceil();
    List<Widget> animalPages = new List();

    for (var i = 0; i < animalPagesNum; i++) {
      animalPages.add(Container(
          color: widget.bgColor,
          child: GridView.count(
            shrinkWrap: true,
            primary: true,
            crossAxisCount: widget.columns,
            children: List.generate(widget.rows * widget.columns, (index) {
                if (index + (widget.columns * widget.rows * i) <
                  animalMap.values.toList().length) {
                  return Center(
                    child: FlatButton(
                      padding: EdgeInsets.all(0),
                      child: Center(
                        child: Text(
                          animalMap.values.toList()[
                            index + (widget.columns * widget.rows * i)],
                          style: TextStyle(fontSize: 18.0),
                        ),
                      ),
                      onPressed: () {
                        widget.onEmojiSelected(
                          Emoji(
                            name: animalMap.keys.toList()[
                              index + (widget.columns * widget.rows * i)],
                            emoji: animalMap.values.toList()[
                              index + (widget.columns * widget.rows * i)]),
                          widget.selectedCategory);
                      },
                  ));
                } else {
                  return Container();
                }
            }),
          ),
      ));
    }

    foodPagesNum = (foodMap.values.toList().length / (widget.rows * widget.columns)).ceil();
    List<Widget> foodPages = new List();

    for (var i = 0; i < foodPagesNum; i++) {
      foodPages.add(Container(
          color: widget.bgColor,
          child: GridView.count(
            shrinkWrap: true,
            primary: true,
            crossAxisCount: widget.columns,
            children: List.generate(widget.rows * widget.columns, (index) {
                if (index + (widget.columns * widget.rows * i) <
                  foodMap.values.toList().length) {
                  return Center(
                    child: FlatButton(
                      padding: EdgeInsets.all(0),
                      child: Center(
                        child: Text(
                          foodMap.values.toList()[
                            index + (widget.columns * widget.rows * i)],
                          style: TextStyle(fontSize: 18.0),
                        ),
                      ),
                      onPressed: () {
                        widget.onEmojiSelected(
                          Emoji(
                            name: foodMap.keys.toList()[
                              index + (widget.columns * widget.rows * i)],
                            emoji: foodMap.values.toList()[
                              index + (widget.columns * widget.rows * i)]),
                          widget.selectedCategory);
                      },
                  ));
                } else {
                  return Container();
                }
            }),
          ),
      ));
    }

    travelPagesNum = (travelMap.values.toList().length / (widget.rows * widget.columns)).ceil();
    List<Widget> travelPages = new List();

    for (var i = 0; i < travelPagesNum; i++) {
      travelPages.add(Container(
          color: widget.bgColor,
          child: GridView.count(
            shrinkWrap: true,
            primary: true,
            crossAxisCount: widget.columns,
            children: List.generate(widget.rows * widget.columns, (index) {
                if (index + (widget.columns * widget.rows * i) <
                  travelMap.values.toList().length) {
                  return Center(
                    child: FlatButton(
                      padding: EdgeInsets.all(0),
                      child: Center(
                        child: Text(
                          travelMap.values.toList()[
                            index + (widget.columns * widget.rows * i)],
                          style: TextStyle(fontSize: 18.0),
                        ),
                      ),
                      onPressed: () {
                        widget.onEmojiSelected(
                          Emoji(
                            name: travelMap.keys.toList()[
                              index + (widget.columns * widget.rows * i)],
                            emoji: travelMap.values.toList()[
                              index + (widget.columns * widget.rows * i)]),
                          widget.selectedCategory);
                      },
                  ));
                } else {
                  return Container();
                }
            }),
          ),
      ));
    }

    activityPagesNum = (activityMap.values.toList().length / (widget.rows * widget.columns)).ceil();
    List<Widget> activityPages = new List();

    for (var i = 0; i < activityPagesNum; i++) {
      activityPages.add(Container(
          color: widget.bgColor,
          child: GridView.count(
            shrinkWrap: true,
            primary: true,
            crossAxisCount: widget.columns,
            children: List.generate(widget.rows * widget.columns, (index) {
                if (index + (widget.columns * widget.rows * i) <
                  activityMap.values.toList().length) {
                  String emojiTxt = activityMap.values
                  .toList()[index + (widget.columns * widget.rows * i)];
                  return Center(
                    child: FlatButton(
                      padding: EdgeInsets.all(0),
                      child: Center(
                        child: Text(
                          activityMap.values.toList()[
                            index + (widget.columns * widget.rows * i)],
                          style: TextStyle(fontSize: 18.0),
                        ),
                      ),
                      onPressed: () {
                        widget.onEmojiSelected(
                          Emoji(
                            name: activityMap.keys.toList()[
                              index + (widget.columns * widget.rows * i)],
                            emoji: activityMap.values.toList()[
                              index + (widget.columns * widget.rows * i)]),
                          widget.selectedCategory);
                      },
                  ));
                } else {
                  return Container();
                }
            }),
          ),
      ));
    }

    objectPagesNum = (objectMap.values.toList().length / (widget.rows * widget.columns)).ceil();
    List<Widget> objectPages = new List();

    for (var i = 0; i < objectPagesNum; i++) {
      objectPages.add(Container(
          color: widget.bgColor,
          child: GridView.count(
            shrinkWrap: true,
            primary: true,
            crossAxisCount: widget.columns,
            children: List.generate(widget.rows * widget.columns, (index) {
                if (index + (widget.columns * widget.rows * i) <
                  objectMap.values.toList().length) {
                  return Center(
                    child: FlatButton(
                      padding: EdgeInsets.all(0),
                      child: Center(
                        child: Text(
                          objectMap.values.toList()[
                            index + (widget.columns * widget.rows * i)],
                          style: TextStyle(fontSize: 18.0),
                        ),
                      ),
                      onPressed: () {
                        widget.onEmojiSelected(
                          Emoji(
                            name: objectMap.keys.toList()[
                              index + (widget.columns * widget.rows * i)],
                            emoji: objectMap.values.toList()[
                              index + (widget.columns * widget.rows * i)]),
                          widget.selectedCategory);
                      },
                  ));
                } else {
                  return Container();
                }
            }),
          ),
      ));
    }

    symbolPagesNum = (symbolMap.values.toList().length / (widget.rows * widget.columns)).ceil();
    List<Widget> symbolPages = new List();

    for (var i = 0; i < symbolPagesNum; i++) {
      symbolPages.add(Container(
          color: widget.bgColor,
          child: GridView.count(
            shrinkWrap: true,
            primary: true,
            crossAxisCount: widget.columns,
            children: List.generate(widget.rows * widget.columns, (index) {
                if (index + (widget.columns * widget.rows * i) <
                  symbolMap.values.toList().length) {
                  return Center(
                    child: FlatButton(
                      padding: EdgeInsets.all(0),
                      child: Center(
                        child: Text(
                          symbolMap.values.toList()[
                            index + (widget.columns * widget.rows * i)],
                          style: TextStyle(fontSize: 18.0),
                        ),
                      ),
                      onPressed: () {
                        widget.onEmojiSelected(
                          Emoji(
                            name: symbolMap.keys.toList()[
                              index + (widget.columns * widget.rows * i)],
                            emoji: symbolMap.values.toList()[
                              index + (widget.columns * widget.rows * i)]),
                          widget.selectedCategory);
                      },
                  ));
                } else {
                  return Container();
                }
            }),
          ),
      ));
    }

    flagPagesNum = (flagMap.values.toList().length / (widget.rows * widget.columns)).ceil();
    List<Widget> flagPages = new List();

    for (var i = 0; i < flagPagesNum; i++) {
      flagPages.add(Container(
          color: widget.bgColor,
          child: GridView.count(
            shrinkWrap: true,
            primary: true,
            crossAxisCount: widget.columns,
            children: List.generate(widget.rows * widget.columns, (index) {
                if (index + (widget.columns * widget.rows * i) <
                  flagMap.values.toList().length) {
                  return Center(
                    child: FlatButton(
                      padding: EdgeInsets.all(0),
                      child: Center(
                        child: Text(
                          flagMap.values.toList()[
                            index + (widget.columns * widget.rows * i)],
                          style: TextStyle(fontSize: 18.0),
                        ),
                      ),
                      onPressed: () {
                        widget.onEmojiSelected(
                          Emoji(
                            name: flagMap.keys.toList()[
                              index + (widget.columns * widget.rows * i)],
                            emoji: flagMap.values.toList()[
                              index + (widget.columns * widget.rows * i)]),
                          widget.selectedCategory);
                      },
                  ));
                } else {
                  return Container();
                }
            }),
          ),
      ));
    }

    pages.clear();
    pages.addAll(smileyPages);
    pages.addAll(animalPages);
    pages.addAll(foodPages);
    pages.addAll(travelPages);
    pages.addAll(activityPages);
    pages.addAll(objectPages);
    pages.addAll(symbolPages);
    pages.addAll(flagPages);

    setState(() {
        loaded = true;
    });
  }

  Widget defaultButton(CategoryIcon categoryIcon) {
    return SizedBox(
      width: 20.0,
      height: 20.0,
      child: Container(
        color: widget.bgColor,
        child: Center(
          child: Icon(
            categoryIcon.icon,
            size: 22,
            color: categoryIcon.color,
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    if (loaded) {
      PageController pageController;
      if (widget.selectedCategory == Category.SMILEYS) {
        pageController = PageController(initialPage: 0);
      } else if (widget.selectedCategory == Category.ANIMALS) {
        pageController = PageController(initialPage: smileyPagesNum);
      } else if (widget.selectedCategory == Category.FOODS) {
        pageController = PageController(
          initialPage: smileyPagesNum +
          animalPagesNum);
      } else if (widget.selectedCategory == Category.TRAVEL) {
        pageController = PageController(
          initialPage: smileyPagesNum +
          animalPagesNum +
          foodPagesNum);
      } else if (widget.selectedCategory == Category.ACTIVITIES) {
        pageController = PageController(
          initialPage: smileyPagesNum +
          animalPagesNum +
          foodPagesNum +
          travelPagesNum);
      } else if (widget.selectedCategory == Category.OBJECTS) {
        pageController = PageController(
          initialPage: smileyPagesNum +
          animalPagesNum +
          foodPagesNum +
          travelPagesNum +
          activityPagesNum);
      } else if (widget.selectedCategory == Category.SYMBOLS) {
        pageController = PageController(
          initialPage: smileyPagesNum +
          animalPagesNum +
          foodPagesNum +
          travelPagesNum +
          activityPagesNum +
          objectPagesNum);
      } else if (widget.selectedCategory == Category.FLAGS) {
        pageController = PageController(
          initialPage: smileyPagesNum +
          animalPagesNum +
          foodPagesNum +
          travelPagesNum +
          activityPagesNum +
          objectPagesNum +
          symbolPagesNum);
      }

      pageController.addListener(() {
          setState(() {});
      });

      final categoryWidth = widget.maxWidth / 8;

      return Column(
        children: <Widget>[
          SizedBox(
            height: 110.0,
            width: widget.maxWidth,
            child: PageView(
              children: pages,
              controller: pageController,
              onPageChanged: (index) {
                if (index < smileyPagesNum) {
                  widget.selectedCategory = Category.SMILEYS;
                } else if (index <
                  smileyPagesNum +
                  animalPagesNum) {
                  widget.selectedCategory = Category.ANIMALS;
                } else if (index <
                  smileyPagesNum +
                  animalPagesNum +
                  foodPagesNum) {
                  widget.selectedCategory = Category.FOODS;
                } else if (index <
                  smileyPagesNum +
                  animalPagesNum +
                  foodPagesNum +
                  travelPagesNum) {
                  widget.selectedCategory = Category.TRAVEL;
                } else if (index <
                  smileyPagesNum +
                  animalPagesNum +
                  foodPagesNum +
                  travelPagesNum +
                  activityPagesNum) {
                  widget.selectedCategory = Category.ACTIVITIES;
                } else if (index <
                  smileyPagesNum +
                  animalPagesNum +
                  foodPagesNum +
                  travelPagesNum +
                  activityPagesNum +
                  objectPagesNum) {
                  widget.selectedCategory = Category.OBJECTS;
                } else if (index <
                  smileyPagesNum +
                  animalPagesNum +
                  foodPagesNum +
                  travelPagesNum +
                  activityPagesNum +
                  objectPagesNum +
                  symbolPagesNum) {
                  widget.selectedCategory = Category.SYMBOLS;
                } else {
                  widget.selectedCategory = Category.FLAGS;
                }
            }),
          ),
          Container(
            color: widget.bgColor,
            height: 6,
            width: widget.maxWidth,
            padding: EdgeInsets.only(top: 4, bottom: 0, right: 2, left: 2),
            child: CustomPaint(
              painter: _ProgressPainter(
                context,
                pageController,
                new Map.fromIterables([
                    Category.SMILEYS,
                    Category.ANIMALS,
                    Category.FOODS,
                    Category.TRAVEL,
                    Category.ACTIVITIES,
                    Category.OBJECTS,
                    Category.SYMBOLS,
                    Category.FLAGS
                  ], [
                    smileyPagesNum,
                    animalPagesNum,
                    foodPagesNum,
                    travelPagesNum,
                    activityPagesNum,
                    objectPagesNum,
                    symbolPagesNum,
                    flagPagesNum
                ]),
                widget.selectedCategory,
                widget.indicatorColor,
                widget.maxWidth,
              )
          )),
          Container(
            height: 40.0,
            color: widget.bgColor,
            child: Row(
              children: <Widget>[
                SizedBox(
                  width: categoryWidth,
                  height: categoryWidth,
                  child: FlatButton(
                    padding: EdgeInsets.all(0),
                    color: widget.selectedCategory == Category.SMILEYS
                    ? Colors.black12
                    : Colors.transparent,
                    shape: RoundedRectangleBorder(
                      borderRadius:
                      BorderRadius.all(Radius.circular(0))),
                    child: Center(
                      child: Icon(
                        widget.categoryIcons.smileyIcon.icon,
                        size: 22,
                        color:
                        widget.selectedCategory == Category.SMILEYS
                        ? widget.categoryIcons.smileyIcon
                        .selectedColor
                        : widget.categoryIcons.smileyIcon.color,
                      ),
                    ),
                    onPressed: () {
                      if (widget.selectedCategory == Category.SMILEYS) {
                        return;
                      }

                      pageController.jumpToPage(0);
                    },
                  )
                ),
                SizedBox(
                  width: categoryWidth,
                  height: categoryWidth,
                  child: FlatButton(
                    padding: EdgeInsets.all(0),
                    color: widget.selectedCategory == Category.ANIMALS
                    ? Colors.black12
                    : Colors.transparent,
                    shape: RoundedRectangleBorder(
                      borderRadius:
                      BorderRadius.all(Radius.circular(0))),
                    child: Center(
                      child: Icon(
                        widget.categoryIcons.animalIcon.icon,
                        size: 22,
                        color:
                        widget.selectedCategory == Category.ANIMALS
                        ? widget.categoryIcons.animalIcon
                        .selectedColor
                        : widget.categoryIcons.animalIcon.color,
                      ),
                    ),
                    onPressed: () {
                      if (widget.selectedCategory == Category.ANIMALS) {
                        return;
                      }

                      pageController.jumpToPage(
                        smileyPagesNum);
                    },
                  )
                ),
                SizedBox(
                  width: categoryWidth,
                  height: categoryWidth,
                  child: FlatButton(
                    padding: EdgeInsets.all(0),
                    color: widget.selectedCategory == Category.FOODS
                    ? Colors.black12
                    : Colors.transparent,
                    shape: RoundedRectangleBorder(
                      borderRadius:
                      BorderRadius.all(Radius.circular(0))),
                    child: Center(
                      child: Icon(
                        widget.categoryIcons.foodIcon.icon,
                        size: 22,
                        color: widget.selectedCategory == Category.FOODS
                        ? widget
                        .categoryIcons.foodIcon.selectedColor
                        : widget.categoryIcons.foodIcon.color,
                      ),
                    ),
                    onPressed: () {
                      if (widget.selectedCategory == Category.FOODS) {
                        return;
                      }

                      pageController.jumpToPage(
                        smileyPagesNum +
                        animalPagesNum);
                    },
                  )
                ),
                SizedBox(
                  width: categoryWidth,
                  height: categoryWidth,
                  child: FlatButton(
                    padding: EdgeInsets.all(0),
                    color: widget.selectedCategory == Category.TRAVEL
                    ? Colors.black12
                    : Colors.transparent,
                    shape: RoundedRectangleBorder(
                      borderRadius:
                      BorderRadius.all(Radius.circular(0))),
                    child: Center(
                      child: Icon(
                        widget.categoryIcons.travelIcon.icon,
                        size: 22,
                        color:
                        widget.selectedCategory == Category.TRAVEL
                        ? widget.categoryIcons.travelIcon
                        .selectedColor
                        : widget.categoryIcons.travelIcon.color,
                      ),
                    ),
                    onPressed: () {
                      if (widget.selectedCategory == Category.TRAVEL) {
                        return;
                      }

                      pageController.jumpToPage(
                        smileyPagesNum +
                        animalPagesNum +
                        foodPagesNum);
                    },
                  )
                ),
                SizedBox(
                  width: categoryWidth,
                  height: categoryWidth,
                  child: FlatButton(
                    padding: EdgeInsets.all(0),
                    color:
                    widget.selectedCategory == Category.ACTIVITIES
                    ? Colors.black12
                    : Colors.transparent,
                    shape: RoundedRectangleBorder(
                      borderRadius:
                      BorderRadius.all(Radius.circular(0))),
                    child: Center(
                      child: Icon(
                        widget.categoryIcons.activityIcon.icon,
                        size: 22,
                        color: widget.selectedCategory ==
                        Category.ACTIVITIES
                        ? widget.categoryIcons.activityIcon
                        .selectedColor
                        : widget.categoryIcons.activityIcon.color,
                      ),
                    ),
                    onPressed: () {
                      if (widget.selectedCategory ==
                        Category.ACTIVITIES) {
                        return;
                      }

                      pageController.jumpToPage(
                        smileyPagesNum +
                        animalPagesNum +
                        foodPagesNum +
                        travelPagesNum);
                    },
                  )
                ),
                SizedBox(
                  width: categoryWidth,
                  height: categoryWidth,
                  child: FlatButton(
                    padding: EdgeInsets.all(0),
                    color: widget.selectedCategory == Category.OBJECTS
                    ? Colors.black12
                    : Colors.transparent,
                    shape: RoundedRectangleBorder(
                      borderRadius:
                      BorderRadius.all(Radius.circular(0))),
                    child: Center(
                      child: Icon(
                        widget.categoryIcons.objectIcon.icon,
                        size: 22,
                        color:
                        widget.selectedCategory == Category.OBJECTS
                        ? widget.categoryIcons.objectIcon
                        .selectedColor
                        : widget.categoryIcons.objectIcon.color,
                      ),
                    ),
                    onPressed: () {
                      if (widget.selectedCategory == Category.OBJECTS) {
                        return;
                      }

                      pageController.jumpToPage(
                        smileyPagesNum +
                        animalPagesNum +
                        foodPagesNum +
                        activityPagesNum +
                        travelPagesNum);
                    },
                  )
                ),
                SizedBox(
                  width: categoryWidth,
                  height: categoryWidth,
                  child: FlatButton(
                    padding: EdgeInsets.all(0),
                    color: widget.selectedCategory == Category.SYMBOLS
                    ? Colors.black12
                    : Colors.transparent,
                    shape: RoundedRectangleBorder(
                      borderRadius:
                      BorderRadius.all(Radius.circular(0))),
                    child: Center(
                      child: Icon(
                        widget.categoryIcons.symbolIcon.icon,
                        size: 22,
                        color:
                        widget.selectedCategory == Category.SYMBOLS
                        ? widget.categoryIcons.symbolIcon
                        .selectedColor
                        : widget.categoryIcons.symbolIcon.color,
                      ),
                    ),
                    onPressed: () {
                      if (widget.selectedCategory == Category.SYMBOLS) {
                        return;
                      }

                      pageController.jumpToPage(
                        smileyPagesNum +
                        animalPagesNum +
                        foodPagesNum +
                        activityPagesNum +
                        travelPagesNum +
                        objectPagesNum);
                    },
                  )
                ),
                SizedBox(
                  width: categoryWidth,
                  height: categoryWidth,
                  child: FlatButton(
                    padding: EdgeInsets.all(0),
                    color: widget.selectedCategory == Category.FLAGS
                    ? Colors.black12
                    : Colors.transparent,
                    shape: RoundedRectangleBorder(
                      borderRadius:
                      BorderRadius.all(Radius.circular(0))),
                    child: Center(
                      child: Icon(
                        widget.categoryIcons.flagIcon.icon,
                        size: 22,
                        color: widget.selectedCategory == Category.FLAGS
                        ? widget
                        .categoryIcons.flagIcon.selectedColor
                        : widget.categoryIcons.flagIcon.color,
                      ),
                    ),
                    onPressed: () {
                      if (widget.selectedCategory == Category.FLAGS) {
                        return;
                      }

                      pageController.jumpToPage(
                        smileyPagesNum +
                        animalPagesNum +
                        foodPagesNum +
                        activityPagesNum +
                        travelPagesNum +
                        objectPagesNum +
                        symbolPagesNum);
                    },
                  )
                ),
              ],
          ))
        ],
      );
    } else {
      return Column(
        children: <Widget>[
          SizedBox(
            height: (widget.maxWidth / widget.columns) * widget.rows,
            width: widget.maxWidth,
            child: Container(
              color: widget.bgColor,
              child: Center(
                child: CircularProgressIndicator(
                  valueColor: AlwaysStoppedAnimation<Color>(widget.progressIndicatorColor),
                ),
              ),
            ),
          ),
          Container(
            height: 6,
            width: widget.maxWidth,
            color: widget.bgColor,
            padding: EdgeInsets.only(top: 4, left: 2, right: 2),
            child: Container(
              color: widget.indicatorColor,
            ),
          ),
          Container(
            height: 40.0,
            child: Row(
              children: <Widget>[
                defaultButton(widget.categoryIcons.smileyIcon),
                defaultButton(widget.categoryIcons.animalIcon),
                defaultButton(widget.categoryIcons.foodIcon),
                defaultButton(widget.categoryIcons.travelIcon),
                defaultButton(widget.categoryIcons.activityIcon),
                defaultButton(widget.categoryIcons.objectIcon),
                defaultButton(widget.categoryIcons.symbolIcon),
                defaultButton(widget.categoryIcons.flagIcon),
              ],
            ),
          )
        ],
      );
    }
  }
}

class _ProgressPainter extends CustomPainter {
  final BuildContext context;
  final PageController pageController;
  final Map<Category, int> pages;
  final Category selectedCategory;
  final Color indicatorColor;
  final double maxWidth;

  _ProgressPainter(this.context, this.pageController, this.pages,
    this.selectedCategory, this.indicatorColor, this.maxWidth);

  @override
  void paint(Canvas canvas, Size size) {
    double offsetInPages = 0;
    if (selectedCategory == Category.SMILEYS) {
      offsetInPages = pageController.offset / maxWidth;
    } else if (selectedCategory == Category.ANIMALS) {
      offsetInPages = (pageController.offset -
        ((pages[Category.SMILEYS]) * maxWidth)) / maxWidth;
    } else if (selectedCategory == Category.FOODS) {
      offsetInPages = (pageController.offset -
        ((pages[Category.SMILEYS] +
            pages[Category.ANIMALS]) *
          maxWidth)) / maxWidth;
    } else if (selectedCategory == Category.TRAVEL) {
      offsetInPages = (pageController.offset -
        ((pages[Category.SMILEYS] +
            pages[Category.ANIMALS] +
            pages[Category.FOODS]) *
          maxWidth)) / maxWidth;
    } else if (selectedCategory == Category.ACTIVITIES) {
      offsetInPages = (pageController.offset -
        ((pages[Category.SMILEYS] +
            pages[Category.ANIMALS] +
            pages[Category.FOODS] +
            pages[Category.TRAVEL]) *
          maxWidth)) / maxWidth;
    } else if (selectedCategory == Category.OBJECTS) {
      offsetInPages = (pageController.offset -
        ((pages[Category.SMILEYS] +
            pages[Category.ANIMALS] +
            pages[Category.FOODS] +
            pages[Category.TRAVEL] +
            pages[Category.ACTIVITIES]) *
          maxWidth)) / maxWidth;
    } else if (selectedCategory == Category.SYMBOLS) {
      offsetInPages = (pageController.offset -
        ((pages[Category.SMILEYS] +
            pages[Category.ANIMALS] +
            pages[Category.FOODS] +
            pages[Category.TRAVEL] +
            pages[Category.ACTIVITIES] +
            pages[Category.OBJECTS]) *
          maxWidth)) / maxWidth;
    } else if (selectedCategory == Category.FLAGS) {
      offsetInPages = (pageController.offset -
        ((pages[Category.SMILEYS] +
            pages[Category.ANIMALS] +
            pages[Category.FOODS] +
            pages[Category.TRAVEL] +
            pages[Category.ACTIVITIES] +
            pages[Category.OBJECTS] +
            pages[Category.SYMBOLS]) *
          maxWidth)) / maxWidth;
    }
    double indicatorPageWidth = size.width / pages[selectedCategory];

    Rect bgRect = Offset(0, 0) & size;

    Rect indicator = Offset(max(0, offsetInPages * indicatorPageWidth), 0) &
    Size(
      indicatorPageWidth -
      max(
        0,
        (indicatorPageWidth +
          (offsetInPages * indicatorPageWidth)) -
        size.width) +
      min(0, offsetInPages * indicatorPageWidth),
      size.height);

    canvas.drawRect(bgRect, Paint()..color = Colors.black12);
    canvas.drawRect(indicator, Paint()..color = indicatorColor);
  }

  @override
  bool shouldRepaint(CustomPainter oldDelegate) {
    return true;
  }
}
