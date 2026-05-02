import React, { useState, useEffect } from "react";
import { deleteOrder, getCustomerOrders } from "@/features/orders/services/orderService";
import { OrderDetailResponse } from "@/features/orders/types/order";
import DeleteConfirmationDialog from "@/shared/components/DeleteConfirmationDialog";
import { Button } from "@/shared/ui/button";
import { Plus, RefreshCw, ShoppingCart } from "lucide-react";
import OrderCard from "./OrderCard";
import OrderFormView from "./OrderFormView";

const CustomerOrderList: React.FC = () => {
  const [orders, setOrders] = useState<OrderDetailResponse[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState<boolean>(false);
  const [editingOrderId, setEditingOrderId] = useState<string | null>(null);
  const [viewingOrderId, setViewingOrderId] = useState<string | null>(null);
  const [formMode, setFormMode] = useState<"view" | "edit" | "create">("create");
  const [deleteDialog, setDeleteDialog] = useState<{ isOpen: boolean; orderId: string; orderNumber: string }>({ isOpen: false, orderId: "", orderNumber: "" });

  const fetchOrders = async (): Promise<void> => {
    try { setLoading(true); setError(null); const data = await getCustomerOrders(); setOrders(data as OrderDetailResponse[]); }
    catch (err) { console.error("Error fetching customer orders:", err); setError("Failed to load customer orders."); }
    finally { setLoading(false); }
  };

  useEffect(() => { fetchOrders(); }, []);

  const handleViewOrder = (orderId: string): void => { setViewingOrderId(orderId); setEditingOrderId(null); setFormMode("view"); setShowForm(true); };
  const handleEditOrder = (orderId: string): void => { setEditingOrderId(orderId); setViewingOrderId(null); setFormMode("edit"); setShowForm(true); };
  const handleDeleteOrder = (orderId: string, orderNumber: string): void => { setDeleteDialog({ isOpen: true, orderId, orderNumber }); };
  const handleAddOrder = (): void => { setEditingOrderId(null); setViewingOrderId(null); setFormMode("create"); setShowForm(true); };
  const handleBackFromForm = (): void => { setShowForm(false); setEditingOrderId(null); setViewingOrderId(null); setFormMode("create"); };
  const handleFormSave = (): void => { setShowForm(false); setEditingOrderId(null); setViewingOrderId(null); setFormMode("create"); fetchOrders(); };
  const handleOrderDeleted = (): void => { fetchOrders(); };
  const handleCloseDeleteDialog = (): void => { setDeleteDialog({ isOpen: false, orderId: "", orderNumber: "" }); };

  if (showForm) {
    return <OrderFormView orderId={editingOrderId || viewingOrderId || undefined} orderType="customer_order" mode={formMode} onSave={handleFormSave} onBack={handleBackFromForm} />;
  }

  return (
    <div className="w-full">
      <div className="flex items-center justify-between mb-5">
        <h2 className="text-[17px] font-semibold text-foreground">Customer Orders</h2>
        <div className="flex gap-2">
          <Button variant="ghost" size="sm" onClick={fetchOrders} disabled={loading} className="h-9 text-[13px]">
            <RefreshCw className={`h-3.5 w-3.5 mr-1.5 ${loading ? "animate-spin" : ""}`} /><span className="hidden sm:inline">Refresh</span>
          </Button>
          <Button size="sm" onClick={handleAddOrder} className="h-9 text-[13px]"><Plus className="h-3.5 w-3.5 mr-1.5" />New Order</Button>
        </div>
      </div>
      {error && <div className="text-[13px] text-destructive bg-destructive/8 rounded-xl px-4 py-3 mb-4">{error}</div>}
      {loading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (<div key={i} className="bg-card rounded-2xl border-[0.5px] border-border/60 p-4 space-y-3 animate-pulse"><div className="flex items-center gap-2.5"><div className="w-8 h-8 rounded-[8px] skeleton" /><div className="space-y-1.5 flex-1"><div className="h-4 w-24 skeleton" /><div className="h-3 w-16 skeleton" /></div></div><div className="grid grid-cols-2 gap-2"><div className="h-3 skeleton" /><div className="h-3 skeleton" /></div></div>))}
        </div>
      ) : orders.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <div className="w-12 h-12 rounded-2xl bg-secondary/60 flex items-center justify-center mb-4"><ShoppingCart className="w-6 h-6 text-muted-foreground/50" /></div>
          <h3 className="text-[17px] font-semibold text-foreground mb-1">No Orders Yet</h3>
          <p className="text-[13px] text-muted-foreground mb-5 max-w-[240px]">Create your first customer order.</p>
          <Button onClick={handleAddOrder}><Plus className="h-4 w-4 mr-2" />Create Your First Order</Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {orders.map((order) => (<OrderCard key={order.id} order={order} onView={handleViewOrder} onEdit={handleEditOrder} onDelete={() => handleDeleteOrder(order.id, order.order_number)} />))}
        </div>
      )}
      <DeleteConfirmationDialog entityLabel="order" isOpen={deleteDialog.isOpen} onClose={handleCloseDeleteDialog} itemLabel={deleteDialog.orderNumber || deleteDialog.orderId} onConfirm={() => deleteOrder(deleteDialog.orderId)} onDeleted={handleOrderDeleted} />
    </div>
  );
};

export default CustomerOrderList;
