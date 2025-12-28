import { create } from "zustand";

export interface DialogState {
  isOpen: boolean
  inputValue: string
  onInputValueChange: (inputValue: string) => void
  handleOpen: () => void
  handleClose: () => void
}


export const useDialogStore = create<DialogState>((set, get) => (
  {
    isOpen: false,
    handleOpen: () => {
      set({ inputValue: "" })
      set({ isOpen: true })
    },
    handleClose: () => {
      set({ isOpen: false })
    },
    inputValue: "",
    onInputValueChange: (inputValue: string) => {
      set({ inputValue });
    },

  }
))


