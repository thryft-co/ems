import React, { useState, useEffect } from "react";
import {
  deleteOrder,
  getCustomerOrders,
} from "@/features/orders/services/orderService";
import { OrderDetailResponse } from "@/features/orders/types/order";
import DeleteConfirmationDialog from "@/shared/components/DeleteConfirmationDialog";
import { Button } from "@/shared/ui/button";
import { Plus, RefreshCw } from "lucide-react";
import OrderCard from "./OrderCard";
import OrderFormView from "./OrderFormView";

const CustomerOrderList: React.FC = () => {
  const [orders, setOrders] = useState<OrderDetailResponse[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState<boolean>(false);
  const [editingOrderId, setEditingOrderId] = useState<string | null>(null);
  const [viewingOrderId, setViewingOrderId] = useState<string | null>(null);
  const [formMode, setFormMode] = useState<"view" | "edit" | "create">(
    "create",
  );
  const [deleteDialog, setDeleteDialog] = useState<{
    isOpen: boolean;
    orderId: string;
    orderNumber: string;
  }>({ isOpen: false, orderId: "", orderNumber: "" });

  const fetchOrders = async (): Promise<void> => {
    try {
      setLoading(true);
      setError(null);
      const data = await getCustomerOrders();
      setOrders(data as OrderDetailResponse[]);
    } catch (err) {
      console.error("Error fetching customer orders:", err);
      setError("Failed to load customer orders. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchOrders();
  }, []);

  const handleViewOrder = (orderId: string): void => {
    setViewingOrderId(orderId);
    setEditingOrderId(null);
    setFormMode("view");
    setShowForm(true);
  };

  const handleEditOrder = (orderId: string): void => {
    setEditingOrderId(orderId);
    setViewingOrderId(null);
    setFormMode("edit");
    setShowForm(true);
  };

  const handleDeleteOrder = (orderId: string, orderNumber: string): void => {
    setDeleteDialog({
      isOpen: true,
      orderId,
      orderNumber,
    });
  };

  const handleAddOrder = (): void => {
    setEditingOrderId(null);
    setViewingOrderId(null);
    setFormMode("create");
    setShowForm(true);
  };

  const handleBackFromForm = (): void => {
    setShowForm(false);
    setEditingOrderId(null);
    setViewingOrderId(null);
    setFormMode("create");
  };

  const handleFormSave = (): void => {
    setShowForm(false);
    setEditingOrderId(null);
    setViewingOrderId(null);
    setFormMode("create");
    fetchOrders();
  };

  const handleOrderDeleted = (): void => {
    fetchOrders();
  };

  const handleCloseDeleteDialog = (): void => {
    setDeleteDialog({ isOpen: false, orderId: "", orderNumber: "" });
  };

  if (showForm) {
    return (
      <OrderFormView
        orderId={editingOrderId || viewingOrderId || undefined}
        orderType="customer_order"
        mode={formMode}
        onSave={handleFormSave}
        onBack={handleBackFromForm}
      />
    );
  }

  return (
    <div className="w-full">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-xl font-semibold">Customer Orders</h2>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={fetchOrders}
            disabled={loading}
          >
            <RefreshCw
              className={`h-4 w-4 mr-2 ${loading ? "animate-spin" : ""}`}
            />
            Refresh
          </Button>
          <Button size="sm" onClick={handleAddOrder}>
            <Plus className="h-4 w-4 mr-2" />
            New Order
          </Button>
        </div>
      </div>

      {error && (
        <div
          className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
          role="alert"
        >
          <p>{error}</p>
        </div>
      )}

      {loading ? (
        <div className="text-center p-8">
          <div className="animate-spin h-8 w-8 border-2 border-primary border-t-transparent rounded-full mx-auto"></div>
          <p className="mt-2 text-muted-foreground">Loading orders...</p>
        </div>
      ) : orders.length === 0 ? (
        <div className="text-center p-8 border rounded-lg">
          <p className="text-muted-foreground">No customer orders found.</p>
          <Button variant="outline" className="mt-4" onClick={handleAddOrder}>
            Create Your First Order
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          {orders.map((order) => (
            <OrderCard
              key={order.id}
              order={order}
              onView={handleViewOrder}
              onEdit={handleEditOrder}
              onDelete={() => handleDeleteOrder(order.id, order.order_number)}
            />
          ))}
        </div>
      )}

      <DeleteConfirmationDialog
        entityLabel="order"
        isOpen={deleteDialog.isOpen}
        onClose={handleCloseDeleteDialog}
        itemLabel={deleteDialog.orderNumber || deleteDialog.orderId}
        onConfirm={() => deleteOrder(deleteDialog.orderId)}
        onDeleted={handleOrderDeleted}
      />
    </div>
  );
};

export default CustomerOrderList;
