import { create } from "zustand";
import { StateEnum, type ContentState } from "../types/state";



export const useStateStore = create<ContentState>((set) => (
  {
    currentState: StateEnum.detail,
    setCurrentState: function (state: StateEnum) {
      set(() => ({ currentState: state }))
    }
  }
))
