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
  onConfirm: () => Promise<unknown>;
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
        className="bg-card !bg-opacity-100 border-[0.5px] border-border/50 shadow-soft-xl rounded-2xl max-w-[320px] sm:max-w-[340px] p-6"
        style={{ backgroundColor: "hsl(var(--card))", opacity: 1 }}
      >
        <AlertDialogHeader className="text-center space-y-2">
          <AlertDialogTitle className="text-[17px] font-semibold">{title}</AlertDialogTitle>
          <AlertDialogDescription className="text-[13px] text-muted-foreground leading-relaxed">
            Are you sure you want to delete{" "}
            {itemLabel ? `${entityLabel} ${itemLabel}` : `this ${entityLabel}`}?
            {" "}This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="flex flex-row gap-3 mt-4">
          <AlertDialogCancel className="flex-1 rounded-[10px] h-11 text-[15px] font-semibold border-0 bg-secondary/60">
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            className="flex-1 rounded-[10px] h-11 text-[15px] font-semibold bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export default DeleteConfirmationDialog;
