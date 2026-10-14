import customtkinter as ctk
from ui.theme import *

ctk.set_appearance_mode("dark")
ctk.set_default_color_theme("dark-blue")

class ZaneMegApp(ctk.CTk):
    def __init__(self):
        super().__init__()
        
        self.title("ZaneMegApp (Modular Edition)")
        self.geometry("900x750")
        self.configure(fg_color=BG_DARK)
        
        # We will build the new modular tabview here
        self._build_header()
        
    def _build_header(self):
        hdr = ctk.CTkFrame(self, fg_color=BG_PANEL, corner_radius=0, height=60)
        hdr.pack(fill="x", side="top")
        
        title_lbl = ctk.CTkLabel(
            hdr, text="ZANE MEGAPP", 
            font=("Helvetica", 18, "bold"), text_color=RED_PRIMARY
        )
        title_lbl.pack(side="left", padx=20, pady=15)
        
        tag = ctk.CTkLabel(hdr, text="V2.0 / MODULAR CORE", font=("Helvetica", 10), text_color=TEXT_MUTED)
        tag.pack(side="left", padx=(0, 20), pady=18)
