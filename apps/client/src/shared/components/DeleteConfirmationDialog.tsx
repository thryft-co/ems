import React from "react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/shared/ui/alert-dialog";

interface DeleteConfirmationDialogProps {
  entityLabel: string;
  isOpen: boolean;
  itemLabel?: string;
  onClose: () => void;
  onConfirm: () => Promise<void>;
  onDeleted?: () => void;
  title?: string;
}

const DeleteConfirmationDialog: React.FC<DeleteConfirmationDialogProps> = ({
  entityLabel,
  isOpen,
  itemLabel,
  onClose,
  onConfirm,
  onDeleted,
  title = "Confirm Deletion",
}) => {
  const handleDelete = async () => {
    try {
      await onConfirm();
      onDeleted?.();
    } catch (error) {
      console.error(`Error deleting ${entityLabel}:`, error);
    } finally {
      onClose();
    }
  };

  return (
    <AlertDialog open={isOpen} onOpenChange={onClose}>
      <AlertDialogContent
        className="bg-white dark:bg-slate-900 !bg-opacity-100 border-2 border-gray-200 shadow-lg rounded-lg"
        style={{ backgroundColor: "white", opacity: 1 }}
      >
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to delete{" "}
            {itemLabel ? `${entityLabel} ${itemLabel}` : `this ${entityLabel}`}?
            {" "}This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            className="bg-red-600 hover:bg-red-700"
          >
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export default DeleteConfirmationDialog;
