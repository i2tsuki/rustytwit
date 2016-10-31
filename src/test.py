#!/usr/bin/env python3
 
from gi.repository import Gtk, Gdk
 
ui_str = """<ui>
    <toolbar name="ToolBar">
        <toolitem action="open"/>
        <toolitem action="save"/>
        <separator/>
        <toolitem action="quit"/>
    </toolbar>
</ui>"""
 
class Win(Gtk.Window):
    """
        GtkUIManager でメニュー、ツールバー、右クリックメニュー
    """
    def __init__(self):
        Gtk.Window.__init__(self)
        # GtkUIManager いつもの処理
        uimanager = Gtk.UIManager()
        accelgroup = uimanager.get_accel_group()
        self.add_accel_group(accelgroup)
        actiongroup = Gtk.ActionGroup("gnome_3_toolbar")
        action_entry = [
            ("open", Gtk.STOCK_OPEN, None, None, "おーぷん", self.on_menu),
            ("save", Gtk.STOCK_SAVE, None, None, "おーぷん", self.on_menu),
            ("quit", Gtk.STOCK_QUIT, None, None, "しゅーりょー", Gtk.main_quit) ]
        actiongroup.add_actions(action_entry)
        uimanager.insert_action_group(actiongroup, 0)
        uimanager.add_ui_from_string(ui_str)
        # ツールバーを取り出す
        toolbar = uimanager.get_widget("/ToolBar")
        # ツールバーにスタイル割付
        style = toolbar.get_style_context()
        # style.add_class(Gtk.STYLE_CLASS_PRIMARY_TOOLBAR)
        # パッキング
        drawingarea = Gtk.DrawingArea()
        vbox = Gtk.Box.new(Gtk.Orientation.VERTICAL, 0)
        vbox.pack_start(toolbar, False, True, 0)
        vbox.pack_start(drawingarea, True, True, 0)
        self.add(vbox)
        # いつもの処理
        self.set_title("GNOME3 Toolbar")
        self.connect("delete-event", Gtk.main_quit)
        self.resize(200, 100)
        self.show_all()
 
    def on_menu(self, action, data=None):
        pass
 
if __name__ == "__main__":
    Win()
    Gtk.main()
